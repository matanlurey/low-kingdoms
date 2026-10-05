//! Direct subprocess invocation, bounded smoke runs, and owned-child cleanup.

use std::fs::File;
use std::path::Path;
use std::process::{Child, Command, ExitStatus, Stdio};
use std::thread;
use std::time::{Duration, Instant};

use crate::{Failure, Result};

#[expect(
    clippy::redundant_pub_crate,
    reason = "Host module entrypoints stay crate-local; pub conflicts with unreachable_pub."
)]
pub(crate) fn run(command: &mut Command) -> Result<()> {
    println!("Running {command:?}");
    let status = command
        .status()
        .map_err(|error| Failure::new(format!("Cannot launch {command:?}: {error}")))?;
    require_success(status, command)
}

#[expect(
    clippy::redundant_pub_crate,
    reason = "Host module entrypoints stay crate-local; pub conflicts with unreachable_pub."
)]
pub(crate) fn capture(command: &mut Command) -> Result<String> {
    let output = command
        .output()
        .map_err(|error| Failure::new(format!("Cannot launch {command:?}: {error}")))?;
    if !output.status.success() {
        return Err(Failure::status(
            format!(
                "{command:?} failed: {}",
                String::from_utf8_lossy(&output.stderr)
            ),
            output.status,
        ));
    }
    String::from_utf8(output.stdout)
        .map_err(|error| Failure::new(format!("{command:?} returned non-UTF-8 text: {error}")))
}

#[expect(
    clippy::redundant_pub_crate,
    reason = "Host module entrypoints stay crate-local; pub conflicts with unreachable_pub."
)]
pub(crate) fn require_success(status: ExitStatus, command: &Command) -> Result<()> {
    if status.success() {
        Ok(())
    } else {
        Err(Failure::status(
            format!("{command:?} exited with {status}"),
            status,
        ))
    }
}

/// Owns the child and its process group, including browser workers on timeout.
#[expect(
    clippy::redundant_pub_crate,
    reason = "Host module entrypoints stay crate-local; pub conflicts with unreachable_pub."
)]
pub(crate) struct OwnedChild {
    child: Child,
}

impl OwnedChild {
    pub(super) fn logged(command: &mut Command, log: &Path) -> Result<Self> {
        let output = File::create(log)
            .map_err(|error| Failure::new(format!("Cannot create {}: {error}", log.display())))?;
        let errors = output.try_clone().map_err(|error| {
            Failure::new(format!("Cannot duplicate log {}: {error}", log.display()))
        })?;
        command
            .stdin(Stdio::null())
            .stdout(Stdio::from(output))
            .stderr(Stdio::from(errors));
        #[cfg(unix)]
        {
            use std::os::unix::process::CommandExt;
            command.process_group(0);
        }
        println!("Running {command:?}; log: {}", log.display());
        let child = command
            .spawn()
            .map_err(|error| Failure::new(format!("Cannot launch {command:?}: {error}")))?;
        Ok(Self { child })
    }

    pub(super) fn try_wait(&mut self) -> Result<Option<ExitStatus>> {
        self.child
            .try_wait()
            .map_err(|error| Failure::new(format!("Cannot inspect owned subprocess: {error}")))
    }

    pub(super) fn wait(&mut self, command: &Command, timeout: Duration) -> Result<()> {
        let started = Instant::now();
        loop {
            if let Some(status) = self.try_wait()? {
                return require_success(status, command);
            }
            if started.elapsed() >= timeout {
                return Err(Failure::new(format!(
                    "{command:?} exceeded {} seconds; its owned processes will be terminated",
                    timeout.as_secs()
                )));
            }
            thread::sleep(Duration::from_millis(50));
        }
    }
}

impl Drop for OwnedChild {
    fn drop(&mut self) {
        // The child has its own process group; cleanup never targets a pre-existing server.
        // Give Playwright time to close its detached browser before forcing group cleanup.
        #[cfg(unix)]
        {
            let group = format!("-{}", self.child.id());
            let signaled = Command::new("kill")
                .args(["-TERM", "--", &group])
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .status()
                .is_ok_and(|status| status.success());
            if signaled {
                let started = Instant::now();
                while started.elapsed() < Duration::from_secs(3) {
                    let active = Command::new("kill")
                        .args(["-0", "--", &group])
                        .stdout(Stdio::null())
                        .stderr(Stdio::null())
                        .status()
                        .is_ok_and(|status| status.success());
                    if !active {
                        break;
                    }
                    thread::sleep(Duration::from_millis(50));
                }
            }
            let _ = Command::new("kill")
                .args(["-KILL", "--", &group])
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .status();
        }
        #[cfg(windows)]
        {
            let _ = Command::new("taskkill")
                .args(["/PID", &self.child.id().to_string(), "/T", "/F"])
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .status();
        }
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::io::Write;

    #[test]
    fn bounded_wait_fails_and_cleans_up_the_owned_child() {
        const CHILD_FLAG: &str = "LOW_KINGDOMS_XTASK_TIMEOUT_TEST_CHILD";
        const HEARTBEAT_PATH: &str = "LOW_KINGDOMS_XTASK_TIMEOUT_TEST_HEARTBEAT";
        let branch = std::env::var(CHILD_FLAG).ok();
        if branch.as_deref() == Some("worker") {
            let mut heartbeat = fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(std::env::var_os(HEARTBEAT_PATH).unwrap())
                .unwrap();
            loop {
                heartbeat.write_all(b".").unwrap();
                thread::sleep(Duration::from_millis(50));
            }
        }
        if branch.as_deref() == Some("child") {
            let mut worker = Command::new(std::env::current_exe().unwrap())
                .args([
                    "--exact",
                    "process::tests::bounded_wait_fails_and_cleans_up_the_owned_child",
                ])
                .env(CHILD_FLAG, "worker")
                .spawn()
                .unwrap();
            assert!(worker.wait().unwrap().success());
            return;
        }
        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let log = std::env::temp_dir().join(format!("low-kingdoms-timeout-{unique}.log"));
        let heartbeat =
            std::env::temp_dir().join(format!("low-kingdoms-timeout-{unique}.heartbeat"));
        let mut command = Command::new(std::env::current_exe().unwrap());
        command
            .args([
                "--exact",
                "process::tests::bounded_wait_fails_and_cleans_up_the_owned_child",
            ])
            .env(CHILD_FLAG, "child")
            .env(HEARTBEAT_PATH, &heartbeat);
        let mut child = OwnedChild::logged(&mut command, &log).unwrap();
        let started = Instant::now();
        while !fs::metadata(&heartbeat).is_ok_and(|metadata| metadata.len() > 0) {
            assert!(started.elapsed() < Duration::from_secs(5));
            assert!(child.try_wait().unwrap().is_none());
            thread::sleep(Duration::from_millis(20));
        }
        let error = child
            .wait(&command, Duration::from_millis(100))
            .unwrap_err();
        assert!(error.to_string().contains("exceeded"));
        assert!(child.try_wait().unwrap().is_none());
        drop(child);
        thread::sleep(Duration::from_millis(100));
        let stopped = fs::metadata(&heartbeat).unwrap().len();
        thread::sleep(Duration::from_millis(100));
        assert_eq!(fs::metadata(&heartbeat).unwrap().len(), stopped);
        fs::remove_file(log).unwrap();
        fs::remove_file(heartbeat).unwrap();
    }

    #[cfg(unix)]
    #[test]
    fn unix_cleanup_waits_for_workers_after_direct_child_exits() {
        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let directory = std::env::temp_dir().join(format!("low-kingdoms-grace-{unique}"));
        fs::create_dir(&directory).unwrap();
        let worker = directory.join("worker.sh");
        fs::write(
            &worker,
            "trap 'sleep 0.3; printf completed > \"$1\"; exit 0' TERM\nprintf ready > \"$2\"\nwhile :; do sleep 0.1; done\n",
        )
        .unwrap();
        let completed = directory.join("completed");
        let ready = directory.join("ready");
        let mut command = Command::new("sh");
        command
            .args([
                "-c",
                "trap 'exit 0' TERM; sh \"$1\" \"$2\" \"$3\" & wait",
                "low-kingdoms-cleanup-test",
            ])
            .arg(&worker)
            .arg(&completed)
            .arg(&ready);
        let child = OwnedChild::logged(&mut command, &directory.join("run.log")).unwrap();
        let started = Instant::now();
        while !ready.exists() {
            assert!(started.elapsed() < Duration::from_secs(5));
            thread::sleep(Duration::from_millis(20));
        }
        drop(child);
        assert_eq!(fs::read_to_string(completed).unwrap(), "completed");
        fs::remove_dir_all(directory).unwrap();
    }

    #[cfg(unix)]
    #[test]
    fn unix_cleanup_forces_workers_that_ignore_term() {
        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let directory = std::env::temp_dir().join(format!("low-kingdoms-force-{unique}"));
        fs::create_dir(&directory).unwrap();
        let heartbeat = directory.join("heartbeat");
        let mut command = Command::new("sh");
        command
            .args([
                "-c",
                "trap '' TERM; while :; do printf . >> \"$1\"; sleep 0.05; done",
                "low-kingdoms-cleanup-test",
            ])
            .arg(&heartbeat);
        let mut child = OwnedChild::logged(&mut command, &directory.join("run.log")).unwrap();
        let started = Instant::now();
        while !fs::metadata(&heartbeat).is_ok_and(|metadata| metadata.len() > 0) {
            assert!(started.elapsed() < Duration::from_secs(5));
            assert!(child.try_wait().unwrap().is_none());
            thread::sleep(Duration::from_millis(20));
        }
        assert!(child.wait(&command, Duration::from_millis(100)).is_err());
        drop(child);
        let stopped = fs::metadata(&heartbeat).unwrap().len();
        thread::sleep(Duration::from_millis(100));
        assert_eq!(fs::metadata(&heartbeat).unwrap().len(), stopped);
        fs::remove_dir_all(directory).unwrap();
    }
}
