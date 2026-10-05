//! Static WASM packaging and real-browser smoke orchestration.

use std::fs;
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use crate::{
    Failure, Options, Project, Result, copy_file, create_directory, process,
    remove_copied_asset_policy,
};

#[expect(
    clippy::redundant_pub_crate,
    reason = "Host module entrypoints stay crate-local; pub conflicts with unreachable_pub."
)]
pub(crate) fn build(project: &Project, output: &Path, test_support: bool) -> Result<()> {
    check_generated_package(&output.join("pkg"))?;
    let bindgen = project.wasm_bindgen()?;
    let mut command = project.cargo(&[
        "build",
        "--locked",
        "--profile",
        "wasm-release",
        "--target",
        "wasm32-unknown-unknown",
        "-p",
        "low-kingdoms",
    ]);
    if test_support {
        command.args(["--features", "test-support"]);
    }
    process::run(&mut command)?;
    create_directory(output)?;
    let staging = OwnedDirectory::create(output)?;
    let generated = staging.0.join("pkg");
    let wasm = project
        .target_dir()?
        .join("wasm32-unknown-unknown/wasm-release/low-kingdoms.wasm");
    process::run(
        Command::new(bindgen)
            .arg(wasm)
            .args([
                "--target",
                "web",
                "--no-typescript",
                "--out-name",
                "low_kingdoms",
                "--out-dir",
            ])
            .arg(&generated),
    )?;
    fs::write(generated.join(".xtask-generated"), GENERATED_PACKAGE_MARKER)
        .map_err(|error| Failure::new(format!("Cannot mark generated WASM package: {error}")))?;
    let package = output.join("pkg");
    if package.exists() {
        // Earlier xtask invocations mark this directory as entirely generated.
        fs::remove_dir_all(&package).map_err(|error| {
            Failure::new(format!(
                "Cannot replace generated package {}: {error}",
                package.display()
            ))
        })?;
    }
    fs::rename(&generated, &package).map_err(|error| {
        Failure::new(format!(
            "Cannot publish generated package {}: {error}",
            package.display()
        ))
    })?;
    for file in ["index.html", "bootstrap.js", "style.css"] {
        copy_file(&project.root.join("web").join(file), &output.join(file))?;
    }
    copy_file(
        &project.root.join("THIRD_PARTY_NOTICES.md"),
        &output.join("THIRD_PARTY_NOTICES.md"),
    )?;
    remove_copied_asset_policy(project, output)?;
    let size = fs::metadata(output.join("pkg/low_kingdoms_bg.wasm"))
        .map_err(|error| Failure::new(format!("Cannot inspect generated WASM: {error}")))?
        .len();
    println!(
        "{} static bundle: {} ({size} WASM bytes)",
        if test_support {
            "Test-support"
        } else {
            "Production"
        },
        output.display()
    );
    Ok(())
}

const GENERATED_PACKAGE_MARKER: &str =
    "Low Kingdoms xtask owns this generated wasm-bindgen directory.\n";

fn check_generated_package(path: &Path) -> Result<()> {
    let metadata = match fs::symlink_metadata(path) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(error) => {
            return Err(Failure::new(format!(
                "Cannot inspect {}: {error}",
                path.display()
            )));
        }
    };
    if !metadata.is_dir() || metadata.is_symlink() {
        return Err(Failure::new(format!(
            "Refusing to replace non-directory {}",
            path.display()
        )));
    }
    let marked = fs::read_to_string(path.join(".xtask-generated"))
        .is_ok_and(|marker| marker == GENERATED_PACKAGE_MARKER);
    if marked {
        return Ok(());
    }
    let empty = fs::read_dir(path)
        .map_err(|error| Failure::new(format!("Cannot inspect {}: {error}", path.display())))?
        .next()
        .is_none();
    if empty {
        Ok(())
    } else {
        Err(Failure::new(format!(
            "Refusing to replace unmarked existing {}; choose --output PATH or remove that generated directory yourself. Only xtask-owned output is replaced.",
            path.display()
        )))
    }
}

#[expect(
    clippy::redundant_pub_crate,
    reason = "Host module entrypoints stay crate-local; pub conflicts with unreachable_pub."
)]
pub(crate) fn dev(project: &Project, options: &Options) -> Result<()> {
    let output = project.output(options.output.as_deref(), "dist/web");
    build(project, &output, false)?;
    let mut server = server_command(project, &output, options.port)?;
    println!(
        "Open http://127.0.0.1:{}/ (rebuild by running dev-web again)",
        options.port
    );
    process::run(&mut server)
}

#[expect(
    clippy::redundant_pub_crate,
    reason = "Host module entrypoints stay crate-local; pub conflicts with unreachable_pub."
)]
pub(crate) fn smoke(project: &Project, options: &Options) -> Result<()> {
    project.require_node()?;
    build(project, &project.root.join("dist/web-test"), true)?;
    build(project, &project.root.join("dist/web"), false)?;
    let evidence = project.output(options.output.as_deref(), "evidence/web");
    create_directory(&evidence)?;
    let directory = OwnedDirectory::create(&project.root.join("dist"))?;
    copy_directory(&project.root.join("dist/web-test"), &directory.0)?;
    copy_directory(
        &project.root.join("dist/web-test"),
        &directory.0.join("low-kingdoms"),
    )?;
    copy_directory(
        &project.root.join("dist/web"),
        &directory.0.join("production"),
    )?;
    let address = SocketAddr::from(([127, 0, 0, 1], options.port));
    // Refuse an occupied port instead of accidentally testing an unrelated server.
    let probe = TcpListener::bind(address).map_err(|error| {
        Failure::new(format!("Cannot bind {address}: {error}; choose --port N"))
    })?;
    drop(probe);
    let mut command = server_command(project, &directory.0, options.port)?;
    let mut server = process::OwnedChild::logged(&mut command, &evidence.join("server.log"))?;
    wait_for_server(&mut server, address)?;
    let base = format!("http://{address}/");
    let mut tests = project.node_cli("@playwright/test/cli.js")?;
    tests
        .args(["test", "--config", "tools/web-tests/playwright.config.mjs"])
        .env("LOW_KINGDOMS_BASE_URL", &base)
        .env("LOW_KINGDOMS_SUBPATH_URL", format!("{base}low-kingdoms/"))
        .env("LOW_KINGDOMS_PRODUCTION_URL", format!("{base}production/"))
        .env("LOW_KINGDOMS_EVIDENCE_DIR", &evidence);
    let mut child = process::OwnedChild::logged(&mut tests, &evidence.join("playwright.log"))?;
    child.wait(&tests, options.timeout.unwrap_or(Duration::from_secs(180)))?;
    println!(
        "Browser checks completed; inspect retained screenshots and traces in {}",
        evidence.display()
    );
    Ok(())
}

fn server_command(project: &Project, directory: &Path, port: u16) -> Result<Command> {
    project.require_node()?;
    // The pinned tool's API avoids CLI interface enumeration forbidden by some sandboxes.
    let mut command = project.command("node");
    command
        .arg(project.root.join("tools/web-tests/serve.mjs"))
        .arg(directory)
        .arg(port.to_string());
    Ok(command)
}

fn wait_for_server(server: &mut process::OwnedChild, address: SocketAddr) -> Result<()> {
    let started = Instant::now();
    while started.elapsed() < Duration::from_secs(10) {
        if let Some(status) = server.try_wait()? {
            return Err(Failure::status(
                format!("Static server exited before readiness: {status}; inspect server.log"),
                status,
            ));
        }
        if TcpStream::connect_timeout(&address, Duration::from_millis(100)).is_ok() {
            return Ok(());
        }
        thread::sleep(Duration::from_millis(50));
    }
    Err(Failure::new(
        "Static server readiness exceeded 10 seconds; inspect server.log",
    ))
}

#[expect(
    clippy::redundant_pub_crate,
    reason = "Host module entrypoints stay crate-local; pub conflicts with unreachable_pub."
)]
pub(crate) fn copy_directory(source: &Path, destination: &Path) -> Result<()> {
    create_directory(destination)?;
    let entries = fs::read_dir(source)
        .map_err(|error| Failure::new(format!("Cannot read {}: {error}", source.display())))?;
    for entry in entries {
        let entry =
            entry.map_err(|error| Failure::new(format!("Cannot read directory entry: {error}")))?;
        let target = destination.join(entry.file_name());
        let kind = entry.file_type().map_err(|error| {
            Failure::new(format!(
                "Cannot inspect {}: {error}",
                entry.path().display()
            ))
        })?;
        if kind.is_dir() {
            copy_directory(&entry.path(), &target)?;
        } else if kind.is_file() {
            copy_file(&entry.path(), &target)?;
        } else {
            return Err(Failure::new(format!(
                "Refusing special file in static bundle: {}",
                entry.path().display()
            )));
        }
    }
    Ok(())
}

struct OwnedDirectory(PathBuf);

impl OwnedDirectory {
    fn create(parent: &Path) -> Result<Self> {
        create_directory(parent)?;
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|error| {
                Failure::new(format!(
                    "Clock cannot identify temporary directory: {error}"
                ))
            })?
            .as_nanos();
        let path = parent.join(format!(".smoke-{}-{timestamp}", std::process::id()));
        fs::create_dir(&path).map_err(|error| {
            Failure::new(format!(
                "Cannot create owned static-server directory {}: {error}",
                path.display()
            ))
        })?;
        Ok(Self(path))
    }
}

impl Drop for OwnedDirectory {
    fn drop(&mut self) {
        if let Err(error) = fs::remove_dir_all(&self.0) {
            eprintln!(
                "Cannot clean owned static-server directory {}: {error}",
                self.0.display()
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn package_replacement_refuses_unmarked_user_files() {
        let directory = OwnedDirectory::create(&std::env::temp_dir()).unwrap();
        let package = directory.0.join("pkg");
        assert!(check_generated_package(&package).is_ok());
        fs::create_dir(&package).unwrap();
        assert!(check_generated_package(&package).is_ok());
        fs::write(package.join("unrelated.txt"), "preserve this").unwrap();
        assert!(check_generated_package(&package).is_err());
        assert_eq!(
            fs::read_to_string(package.join("unrelated.txt")).unwrap(),
            "preserve this"
        );
        fs::write(package.join(".xtask-generated"), GENERATED_PACKAGE_MARKER).unwrap();
        assert!(check_generated_package(&package).is_ok());
    }
}
