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
        #[cfg(unix)]
        {
            let _ = Command::new("kill")
                .args(["-TERM", "--", &format!("-{}", self.child.id())])
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

    #[test]
    fn bounded_wait_fails_and_cleans_up_the_owned_child() {
        const CHILD_FLAG: &str = "LOW_KINGDOMS_XTASK_TIMEOUT_TEST_CHILD";
        if std::env::var_os(CHILD_FLAG).is_some() {
            loop {
                thread::sleep(Duration::from_millis(50));
            }
        }
        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let log = std::env::temp_dir().join(format!("low-kingdoms-timeout-{unique}.log"));
        let mut command = Command::new(std::env::current_exe().unwrap());
        command
            .args([
                "--exact",
                "process::tests::bounded_wait_fails_and_cleans_up_the_owned_child",
            ])
            .env(CHILD_FLAG, "1");
        let mut child = OwnedChild::logged(&mut command, &log).unwrap();
        let error = child
            .wait(&command, Duration::from_millis(100))
            .unwrap_err();
        assert!(error.to_string().contains("exceeded"));
        assert!(child.try_wait().unwrap().is_none());
        drop(child);
        fs::remove_file(log).unwrap();
    }
}
