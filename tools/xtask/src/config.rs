//! Project-root paths and the single tool/version policy.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use serde_json::Value;

use crate::{Failure, Result, process};

#[expect(
    clippy::redundant_pub_crate,
    reason = "Host module entrypoints stay crate-local; pub conflicts with unreachable_pub."
)]
pub(crate) struct Project {
    pub(super) root: PathBuf,
    tools: toml::Value,
}

impl Project {
    pub(super) fn load() -> Result<Self> {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(Path::parent)
            .ok_or_else(|| Failure::new("xtask manifest is not inside tools/xtask"))?
            .to_path_buf();
        let tools = read_toml(&root.join("tools/tool-versions.toml"))?;
        Ok(Self { root, tools })
    }

    pub(super) fn version(&self, name: &str) -> Result<&str> {
        self.tools
            .get("tools")
            .and_then(|tools| tools.get(name))
            .and_then(toml::Value::as_str)
            .ok_or_else(|| Failure::new(format!("tools/tool-versions.toml needs [tools] {name}")))
    }

    pub(super) fn command(&self, program: &str) -> Command {
        let mut command = Command::new(program);
        command.current_dir(&self.root);
        command
    }

    pub(super) fn cargo(&self, args: &[&str]) -> Command {
        let mut command = self.command("cargo");
        command.args(args);
        command
    }

    pub(super) fn metadata(&self) -> Result<Value> {
        let output = process::capture(&mut self.cargo(&[
            "metadata",
            "--locked",
            "--no-deps",
            "--format-version",
            "1",
        ]))?;
        serde_json::from_str(&output)
            .map_err(|error| Failure::new(format!("Invalid cargo metadata: {error}")))
    }

    pub(super) fn target_dir(&self) -> Result<PathBuf> {
        self.metadata()?["target_directory"]
            .as_str()
            .map(PathBuf::from)
            .ok_or_else(|| Failure::new("Cargo metadata has no target_directory"))
    }

    pub(super) fn wasm_version(&self) -> Result<String> {
        let lock = read_toml(&self.root.join("Cargo.lock"))?;
        let packages = lock
            .get("package")
            .and_then(toml::Value::as_array)
            .ok_or_else(|| Failure::new("Cargo.lock has no package list"))?;
        let versions: Vec<_> = packages
            .iter()
            .filter(|package| {
                package.get("name").and_then(toml::Value::as_str) == Some("wasm-bindgen")
            })
            .filter_map(|package| package.get("version").and_then(toml::Value::as_str))
            .collect();
        match versions.as_slice() {
            [version] => Ok((*version).to_owned()),
            _ => Err(Failure::new(
                "Cargo.lock must resolve exactly one wasm-bindgen version; resolve dependencies first",
            )),
        }
    }

    pub(super) fn local_bin(&self, tool: &str, version: &str) -> PathBuf {
        self.root
            .join(".tools")
            .join(format!("{tool}-{version}"))
            .join("bin")
            .join(format!("{tool}{}", std::env::consts::EXE_SUFFIX))
    }

    pub(super) fn wasm_bindgen(&self) -> Result<PathBuf> {
        let version = self.wasm_version()?;
        let binary = self.local_bin("wasm-bindgen", &version);
        let mut command = Command::new(&binary);
        command.arg("--version");
        let actual = process::capture(&mut command).map_err(|error| {
            Failure::new(format!(
                "Project-local wasm-bindgen {version} is required at {}; run cargo xtask setup. {error}",
                binary.display()
            ))
        })?;
        require_version("wasm-bindgen", &actual, &format!("wasm-bindgen {version}"))?;
        Ok(binary)
    }

    pub(super) fn require_node(&self) -> Result<()> {
        let actual = process::capture(self.command("node").arg("--version"))?;
        require_version("Node", &actual, &format!("v{}", self.version("node")?))
    }

    pub(super) fn node_cli(&self, relative: &str) -> Result<Command> {
        self.require_node()?;
        let script = self
            .root
            .join("tools/web-tests/node_modules")
            .join(relative);
        if !script.is_file() {
            return Err(Failure::new(format!(
                "Missing {}; run cargo xtask setup (or npm ci --prefix tools/web-tests)",
                script.display()
            )));
        }
        let mut command = self.command("node");
        command.arg(script);
        Ok(command)
    }

    pub(super) fn output(&self, override_path: Option<&Path>, default: &str) -> PathBuf {
        override_path.map_or_else(|| self.root.join(default), |path| self.root.join(path))
    }
}

#[expect(
    clippy::redundant_pub_crate,
    reason = "Host module entrypoints stay crate-local; pub conflicts with unreachable_pub."
)]
pub(crate) fn read_toml(path: &Path) -> Result<toml::Value> {
    let source = fs::read_to_string(path)
        .map_err(|error| Failure::new(format!("Cannot read {}: {error}", path.display())))?;
    toml::from_str(&source)
        .map_err(|error| Failure::new(format!("Invalid TOML in {}: {error}", path.display())))
}

#[expect(
    clippy::redundant_pub_crate,
    reason = "Host module entrypoints stay crate-local; pub conflicts with unreachable_pub."
)]
pub(crate) fn require_version(tool: &str, actual: &str, expected: &str) -> Result<()> {
    if actual.trim() == expected {
        Ok(())
    } else {
        Err(Failure::new(format!(
            "{tool} version mismatch: expected {expected}, found {}; use tools/tool-versions.toml and cargo xtask setup",
            actual.trim()
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_unmatched_cli_and_accepts_newline() {
        assert!(
            require_version("wasm-bindgen", "wasm-bindgen 0.2.1\n", "wasm-bindgen 0.2.1").is_ok()
        );
        let error = require_version("Node", "v24.1.0", "v24.2.0").unwrap_err();
        assert!(error.to_string().contains("version mismatch"));
    }

    #[test]
    fn resolves_root_and_reads_real_tool_policy_and_lock() {
        let project = Project::load().unwrap();
        assert!(project.root.join("apps/game/Cargo.toml").is_file());
        assert!(!project.version("rust").unwrap().is_empty());
        assert!(!project.wasm_version().unwrap().is_empty());
        assert!(project.version("not-a-tool").is_err());
    }
}
