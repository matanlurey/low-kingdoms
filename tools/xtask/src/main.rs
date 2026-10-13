//! Portable host-only build, verification, and rendered-evidence commands.

mod config;
mod process;
mod structure;
mod web;

use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode, ExitStatus};
use std::time::Duration;

use config::Project;

type Result<T> = std::result::Result<T, Failure>;

#[derive(Debug)]
struct Failure {
    message: String,
    code: u8,
}

impl Failure {
    fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            code: 1,
        }
    }

    fn status(message: String, status: ExitStatus) -> Self {
        Self {
            message,
            code: status
                .code()
                .and_then(|code| u8::try_from(code).ok())
                .filter(|code| *code != 0)
                .unwrap_or(1),
        }
    }
}

impl fmt::Display for Failure {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.message)
    }
}

#[derive(Debug, PartialEq, Eq)]
struct Options {
    command: String,
    port: u16,
    output: Option<PathBuf>,
    timeout: Option<Duration>,
}

impl Options {
    fn parse(args: impl IntoIterator<Item = String>) -> Result<Self> {
        let mut args = args.into_iter();
        let command = args.next().unwrap_or_else(|| "help".to_owned());
        let mut options = Self {
            command,
            port: 4173,
            output: None,
            timeout: None,
        };
        while let Some(argument) = args.next() {
            let value = args
                .next()
                .ok_or_else(|| Failure::new(format!("{argument} requires a value")))?;
            match argument.as_str() {
                "--port" => {
                    options.port = value
                        .parse()
                        .map_err(|error| Failure::new(format!("Invalid port {value}: {error}")))?;
                    if options.port == 0 {
                        return Err(Failure::new("--port must be between 1 and 65535"));
                    }
                }
                "--output" => options.output = Some(PathBuf::from(value)),
                "--timeout" => {
                    let seconds: u64 = value.parse().map_err(|error| {
                        Failure::new(format!("Invalid timeout {value}: {error}"))
                    })?;
                    if seconds == 0 {
                        return Err(Failure::new("--timeout must be positive seconds"));
                    }
                    options.timeout = Some(Duration::from_secs(seconds));
                }
                _ => return Err(Failure::new(format!("Unknown option {argument}"))),
            }
        }
        Ok(options)
    }
}

fn main() -> ExitCode {
    match execute() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("xtask: {error}");
            ExitCode::from(error.code)
        }
    }
}

fn execute() -> Result<()> {
    let options = Options::parse(std::env::args().skip(1))?;
    let project = Project::load()?;
    match options.command.as_str() {
        "help" | "--help" | "-h" => {
            println!(
                "cargo xtask <doctor|setup|dev|dev-web|build|build-web|fmt|fmt-check|clippy|test|doc|check|check-web|smoke-native|smoke-web|check-all|structure> [--port N] [--output PATH] [--timeout SECONDS]"
            );
            Ok(())
        }
        "doctor" => doctor(&project),
        "setup" => setup(&project),
        "dev" => process::run(
            project
                .cargo(&["run", "--locked", "-p", "low-kingdoms"])
                .env("BEVY_ASSET_ROOT", &project.root),
        ),
        "dev-web" => web::dev(&project, &options),
        "build" => build_native(&project, &options),
        "build-web" => web::build(
            &project,
            &project.output(options.output.as_deref(), "dist/web"),
            false,
        ),
        "fmt" => formatting(&project, false),
        "fmt-check" => formatting(&project, true),
        "clippy" => clippy(&project),
        "test" => tests(&project),
        "doc" => docs(&project),
        "structure" => structure::check(&project),
        "check" => check(&project),
        "check-web" => check_web(&project),
        "smoke-native" => smoke_native(&project, &options),
        "smoke-web" => web::smoke(&project, &options),
        "check-all" => {
            check(&project)?;
            check_web(&project)?;
            let mut native = options.clone_for("smoke-native");
            native.output = options.output.as_ref().map(|path| path.join("native"));
            smoke_native(&project, &native)?;
            let mut browser = options.clone_for("smoke-web");
            browser.output = options.output.as_ref().map(|path| path.join("web"));
            web::smoke(&project, &browser)
        }
        name => Err(Failure::new(format!(
            "Unknown command {name}; use cargo xtask help"
        ))),
    }
}

impl Options {
    fn clone_for(&self, command: &str) -> Self {
        Self {
            command: command.to_owned(),
            port: self.port,
            output: self.output.clone(),
            timeout: self.timeout,
        }
    }
}

fn doctor(project: &Project) -> Result<()> {
    println!("Repository: {}", project.root.display());
    let mut failures = Vec::new();
    for (program, argument) in [
        ("rustc", "--version"),
        ("cargo", "--version"),
        ("rustfmt", "--version"),
        ("clippy-driver", "--version"),
        ("node", "--version"),
        (if cfg!(windows) { "npm.cmd" } else { "npm" }, "--version"),
    ] {
        match process::capture(project.command(program).arg(argument)) {
            Ok(version) => println!("{program}: {}", version.trim()),
            Err(error) => failures.push(error.to_string()),
        }
    }
    for (name, result) in [
        ("Rust policy", rust_version(project)),
        ("Node policy", project.require_node()),
        ("wasm-bindgen policy", project.wasm_bindgen().map(|_| ())),
    ] {
        match result {
            Ok(()) => println!("{name}: matched"),
            Err(error) => failures.push(error.to_string()),
        }
    }
    match process::capture(
        project
            .command("rustup")
            .args(["target", "list", "--installed"]),
    ) {
        Ok(targets) => {
            println!("Installed targets:\n{}", targets.trim());
            if !targets.lines().any(|line| line == "wasm32-unknown-unknown") {
                failures.push("Missing wasm32-unknown-unknown; run cargo xtask setup".to_owned());
            }
        }
        Err(error) => failures.push(error.to_string()),
    }
    for (package, script, expected) in [
        (
            "Playwright",
            "@playwright/test/cli.js",
            format!("Version {}", project.version("playwright")?),
        ),
        (
            "Prettier",
            "prettier/bin/prettier.cjs",
            project.version("prettier")?.to_owned(),
        ),
    ] {
        match project
            .node_cli(script)
            .and_then(|mut command| process::capture(command.arg("--version")))
        {
            Ok(version) => {
                println!("{package}: {}", version.trim());
                if let Err(error) = config::require_version(package, &version, &expected) {
                    failures.push(error.to_string());
                }
            }
            Err(error) => failures.push(error.to_string()),
        }
    }
    for (name, relative) in [(
        "http-server",
        "tools/web-tests/node_modules/http-server/package.json",
    )] {
        match installed_package(project, relative, project.version(name)?) {
            Ok(()) => println!(
                "{name}: {} (pinned static-server API)",
                project.version(name)?
            ),
            Err(error) => failures.push(error.to_string()),
        }
    }
    let just = project.local_bin("just", project.version("just")?);
    match process::capture(Command::new(&just).arg("--version")) {
        Ok(version) => {
            println!("Just: {}", version.trim());
            if let Err(error) = config::require_version(
                "Just",
                &version,
                &format!("just {}", project.version("just")?),
            ) {
                failures.push(error.to_string());
            }
        }
        Err(error) => failures.push(format!(
            "Project-local Just is missing; run cargo xtask setup. {error}"
        )),
    }
    println!(
        "Native smoke requires a desktop display and a working graphics adapter. Linux needs X11/Wayland, the XKB X11 companion runtime (libxkbcommon-x11-0 on Ubuntu), and Vulkan or OpenGL drivers (software rendering is acceptable). Chromium also requires its OS libraries; setup never installs privileged OS packages. See docs/development.md."
    );
    if failures.is_empty() {
        Ok(())
    } else {
        Err(Failure::new(failures.join("\n")))
    }
}

fn installed_package(project: &Project, relative: &str, expected: &str) -> Result<()> {
    let path = project.root.join(relative);
    let source = fs::read_to_string(&path).map_err(|error| {
        Failure::new(format!(
            "Cannot read installed package {}: {error}; run cargo xtask setup",
            path.display()
        ))
    })?;
    let package: serde_json::Value = serde_json::from_str(&source).map_err(|error| {
        Failure::new(format!(
            "Invalid installed package {}: {error}",
            path.display()
        ))
    })?;
    let actual = package["version"].as_str().ok_or_else(|| {
        Failure::new(format!(
            "Installed package {} has no version",
            path.display()
        ))
    })?;
    config::require_version(relative, actual, expected)
}

fn rust_version(project: &Project) -> Result<()> {
    let actual = process::capture(project.command("rustc").arg("--version"))?;
    let expected = project.version("rust")?;
    if actual.split_whitespace().nth(1) == Some(expected) {
        Ok(())
    } else {
        Err(Failure::new(format!(
            "Rust mismatch: expected {expected}, found {}",
            actual.trim()
        )))
    }
}

fn setup(project: &Project) -> Result<()> {
    process::run(project.command("rustup").args([
        "toolchain",
        "install",
        project.version("rust")?,
        "--profile",
        "minimal",
        "--component",
        "rustfmt",
        "--component",
        "clippy",
        "--target",
        "wasm32-unknown-unknown",
    ]))?;
    project.require_node()?;
    for (package, binary, version) in [
        ("just", "just", project.version("just")?.to_owned()),
        ("wasm-bindgen-cli", "wasm-bindgen", project.wasm_version()?),
    ] {
        let local = project.local_bin(binary, &version);
        let install_root = project
            .root
            .join(".tools")
            .join(format!("{binary}-{version}"));
        let expected = format!("{binary} {version}");
        let mut version_command = Command::new(&local);
        version_command.arg("--version");
        let installed =
            process::capture(&mut version_command).is_ok_and(|actual| actual.trim() == expected);
        if !installed {
            process::run(
                project
                    .command("cargo")
                    .args([
                        "install",
                        package,
                        "--version",
                        &version,
                        "--locked",
                        "--root",
                    ])
                    .arg(&install_root),
            )?;
        }
    }
    let npm = if cfg!(windows) { "npm.cmd" } else { "npm" };
    process::run(
        project
            .command(npm)
            .args(["ci", "--prefix", "tools/web-tests"]),
    )?;
    process::run(
        project
            .node_cli("@playwright/test/cli.js")?
            .args(["install", "chromium"]),
    )?;
    println!(
        "Setup complete. Project-local Just is in {}. cargo xtask commands work without adding it to PATH.",
        project
            .local_bin("just", project.version("just")?)
            .display()
    );
    Ok(())
}

fn formatting(project: &Project, check_only: bool) -> Result<()> {
    let mut rust = project.cargo(&["fmt", "--all"]);
    if check_only {
        rust.args(["--", "--check"]);
    }
    process::run(&mut rust)?;
    let mut text = project.node_cli("prettier/bin/prettier.cjs")?;
    text.args([
        if check_only { "--check" } else { "--write" },
        "--ignore-path",
        ".gitignore",
    ]);
    text.args([
        "*.md",
        "docs/**/*.md",
        ".agents/**/*.md",
        ".github/**/*.yml",
        "crates/*/AGENTS.md",
        "assets/README.md",
        "web/**/*.{html,js,css}",
        "tools/web-tests/**/*.{json,mjs,js}",
    ]);
    process::run(&mut text)
}

fn clippy(project: &Project) -> Result<()> {
    for features in [None, Some("test-support")] {
        let mut command = project.cargo(&["clippy", "--locked", "--workspace", "--all-targets"]);
        if let Some(features) = features {
            command.args(["--features", features]);
        }
        command.args(["--", "-D", "warnings"]);
        process::run(&mut command)?;
    }
    Ok(())
}

fn tests(project: &Project) -> Result<()> {
    process::run(&mut project.cargo(&["test", "--locked", "--workspace"]))?;
    process::run(&mut project.cargo(&[
        "test",
        "--locked",
        "-p",
        "low-kingdoms-client",
        "--features",
        "test-support",
    ]))
}

fn docs(project: &Project) -> Result<()> {
    process::run(
        project
            .cargo(&["doc", "--locked", "--workspace", "--no-deps"])
            .env("RUSTDOCFLAGS", "-D warnings"),
    )
}

fn check(project: &Project) -> Result<()> {
    formatting(project, true)?;
    structure::check(project)?;
    clippy(project)?;
    tests(project)?;
    docs(project)
}

fn check_web(project: &Project) -> Result<()> {
    for features in [None, Some("test-support")] {
        for action in ["check", "clippy"] {
            let mut command = project.cargo(&[
                action,
                "--locked",
                "--target",
                "wasm32-unknown-unknown",
                "-p",
                "low-kingdoms",
                "-p",
                "low-kingdoms-client",
                "-p",
                "low-kingdoms-rules",
                "-p",
                "low-kingdoms-battle",
            ]);
            if let Some(features) = features {
                command.args(["--features", features]);
            }
            if action == "clippy" {
                command.args(["--", "-D", "warnings"]);
            }
            process::run(&mut command)?;
        }
    }
    Ok(())
}

fn build_native(project: &Project, options: &Options) -> Result<()> {
    process::run(&mut project.cargo(&["build", "--locked", "--release", "-p", "low-kingdoms"]))?;
    let output = project.output(options.output.as_deref(), "dist/native");
    create_directory(&output)?;
    let filename = format!("low-kingdoms{}", std::env::consts::EXE_SUFFIX);
    copy_file(
        &project.target_dir()?.join("release").join(&filename),
        &output.join(filename),
    )?;
    copy_file(
        &project.root.join("THIRD_PARTY_NOTICES.md"),
        &output.join("THIRD_PARTY_NOTICES.md"),
    )?;
    remove_copied_asset_policy(project, &output)?;
    println!("Native release distribution: {}", output.display());
    Ok(())
}

fn smoke_native(project: &Project, options: &Options) -> Result<()> {
    process::run(&mut project.cargo(&[
        "build",
        "--locked",
        "-p",
        "low-kingdoms",
        "--features",
        "test-support",
    ]))?;
    let output = project.output(options.output.as_deref(), "evidence/native");
    create_directory(&output)?;
    let binary = project
        .target_dir()?
        .join("debug")
        .join(format!("low-kingdoms{}", std::env::consts::EXE_SUFFIX));
    for scene in ["title", "mulligan", "battle", "results"] {
        capture_native_scene(project, options, &binary, &output, scene)?;
    }
    println!(
        "Native title, mulligan, deployed battlefield, and results captures completed in {}. Inspect each image for readable text, controls, and clipping; capture completion alone is not visual approval.",
        output.display()
    );
    Ok(())
}

fn capture_native_scene(
    project: &Project,
    options: &Options,
    binary: &Path,
    output: &Path,
    scene: &str,
) -> Result<()> {
    let screenshot = output.join(format!("{scene}.png"));
    if screenshot.exists() {
        fs::remove_file(&screenshot)
            .map_err(|error| Failure::new(format!("Cannot remove prior screenshot: {error}")))?;
    }
    // Test-support alone honors this override. Every scene owns a fresh active slot,
    // so smoke runs cannot read or replace a developer's normal resumable battle.
    let data_directory = output.join("save-fixtures").join(scene);
    if data_directory.exists() {
        fs::remove_dir_all(&data_directory).map_err(|error| {
            Failure::new(format!("Cannot clear prior native fixture slot: {error}"))
        })?;
    }
    create_directory(&data_directory)?;
    let log = output.join(format!("{scene}.log"));
    let mut command = Command::new(binary);
    command
        .current_dir(&project.root)
        .env("BEVY_ASSET_ROOT", &project.root)
        .env("LOW_KINGDOMS_TEST_DATA_DIR", &data_directory)
        .arg("--screenshot")
        .arg(&screenshot)
        .arg("--scene")
        .arg(scene);
    let mut child = process::OwnedChild::logged(&mut command, &log)?;
    child.wait(&command, options.timeout.unwrap_or(Duration::from_secs(60)))?;
    let bytes = fs::read(&screenshot).map_err(|error| {
        Failure::new(format!(
            "Renderer exited without capture {}: {error}; inspect {}",
            screenshot.display(),
            log.display()
        ))
    })?;
    if bytes.len() < 1024 || !bytes.starts_with(b"\x89PNG\r\n\x1a\n") {
        return Err(Failure::new(format!(
            "Native renderer capture is missing or invalid; inspect {}",
            log.display()
        )));
    }
    println!("Native {scene} capture completed: {}", screenshot.display());
    Ok(())
}

fn create_directory(path: &Path) -> Result<()> {
    fs::create_dir_all(path)
        .map_err(|error| Failure::new(format!("Cannot create {}: {error}", path.display())))
}

fn copy_file(source: &Path, destination: &Path) -> Result<()> {
    fs::copy(source, destination).map(|_| ()).map_err(|error| {
        Failure::new(format!(
            "Cannot copy {} to {}: {error}",
            source.display(),
            destination.display()
        ))
    })
}

fn remove_copied_asset_policy(project: &Project, output: &Path) -> Result<()> {
    // The only scaffold font is embedded; the repository's policy is not a runtime asset.
    let assets = output.join("assets");
    let readme = assets.join("README.md");
    for path in [&assets, &readme] {
        match fs::symlink_metadata(path) {
            Ok(metadata) if metadata.is_symlink() => {
                return Err(Failure::new(format!(
                    "Refusing to clean linked asset-policy output {}; choose a fresh --output directory",
                    path.display()
                )));
            }
            Ok(_) => {}
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
            Err(error) => {
                return Err(Failure::new(format!(
                    "Cannot inspect prior asset-policy output {}: {error}",
                    path.display()
                )));
            }
        }
    }
    if readme.is_file() {
        let policy = fs::read(project.root.join("assets/README.md")).map_err(|error| {
            Failure::new(format!("Cannot read maintained asset policy: {error}"))
        })?;
        let existing = fs::read(&readme).map_err(|error| {
            Failure::new(format!("Cannot inspect {}: {error}", readme.display()))
        })?;
        if existing == policy {
            fs::remove_file(&readme).map_err(|error| {
                Failure::new(format!(
                    "Cannot remove copied asset policy {}: {error}",
                    readme.display()
                ))
            })?;
        }
    }
    if assets.is_dir()
        && fs::read_dir(&assets)
            .map_err(|error| Failure::new(format!("Cannot inspect {}: {error}", assets.display())))?
            .next()
            .is_none()
    {
        fs::remove_dir(&assets).map_err(|error| {
            Failure::new(format!(
                "Cannot remove empty generated asset directory: {error}"
            ))
        })?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn argument_parser_preserves_paths_as_arguments() {
        let options = Options::parse(
            [
                "smoke-web",
                "--output",
                "evidence/my output",
                "--port",
                "9090",
                "--timeout",
                "45",
            ]
            .map(str::to_owned),
        )
        .unwrap();
        assert_eq!(options.output, Some(PathBuf::from("evidence/my output")));
        assert_eq!(options.port, 9090);
        assert_eq!(options.timeout, Some(Duration::from_secs(45)));
    }

    #[test]
    fn argument_parser_rejects_missing_invalid_and_unknown_values() {
        for args in [
            vec!["dev-web", "--port"],
            vec!["dev-web", "--port", "0"],
            vec!["smoke-native", "--timeout", "0"],
            vec!["dev", "--unknown", "value"],
        ] {
            assert!(Options::parse(args.into_iter().map(str::to_owned)).is_err());
        }
    }

    #[test]
    fn failed_operation_cannot_return_successful_child_exit_code() {
        #[cfg(unix)]
        use std::os::unix::process::ExitStatusExt;
        #[cfg(windows)]
        use std::os::windows::process::ExitStatusExt;
        let status = ExitStatus::from_raw(0);
        let failure = Failure::status("Required server exited before readiness".to_owned(), status);
        assert_eq!(failure.code, 1);
    }
}
