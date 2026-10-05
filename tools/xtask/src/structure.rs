//! Checks implemented package boundaries and repository-local instructions.

use std::fs;
use std::path::Path;

use crate::config::{Project, read_toml};
use crate::{Failure, Result};

#[expect(
    clippy::redundant_pub_crate,
    reason = "Host module entrypoints stay crate-local; pub conflicts with unreachable_pub."
)]
pub(crate) fn check(project: &Project) -> Result<()> {
    check_packages(project)?;
    check_documents(project)?;
    check_browser_pins(project)?;
    println!("Repository structure and local skill metadata passed.");
    Ok(())
}

fn check_packages(project: &Project) -> Result<()> {
    let metadata = project.metadata()?;
    let packages = metadata["packages"]
        .as_array()
        .ok_or_else(|| Failure::new("Cargo metadata did not contain packages"))?;
    let members = metadata["workspace_members"]
        .as_array()
        .ok_or_else(|| Failure::new("Cargo metadata did not contain workspace_members"))?;
    if packages.len() != 3 || members.len() != 3 {
        return Err(Failure::new(
            "Scaffold must have exactly its three implemented packages",
        ));
    }
    for (name, relative) in [
        ("low-kingdoms", "apps/game/Cargo.toml"),
        ("low-kingdoms-client", "crates/client/Cargo.toml"),
        ("xtask", "tools/xtask/Cargo.toml"),
    ] {
        let package = packages
            .iter()
            .find(|package| package["name"].as_str() == Some(name))
            .ok_or_else(|| Failure::new(format!("Missing workspace package {name}")))?;
        if package["publish"]
            .as_array()
            .is_none_or(|publish| !publish.is_empty())
        {
            return Err(Failure::new(format!("{name} must set publish = false")));
        }
        let manifest_path = project.root.join(relative);
        let reported_path = package["manifest_path"]
            .as_str()
            .ok_or_else(|| Failure::new(format!("{name} has no manifest_path")))?;
        if Path::new(reported_path) != manifest_path {
            return Err(Failure::new(format!("{name} is outside {relative}")));
        }
        let manifest = read_toml(&manifest_path)?;
        if manifest
            .get("lints")
            .and_then(|lints| lints.get("workspace"))
            .and_then(toml::Value::as_bool)
            != Some(true)
        {
            return Err(Failure::new(format!("{name} must inherit workspace lints")));
        }
        let dependencies = package["dependencies"]
            .as_array()
            .ok_or_else(|| Failure::new(format!("{name} has no dependencies")))?;
        for dependency in dependencies {
            let dependency_name = dependency["name"].as_str().unwrap_or("");
            if (name != "xtask" && dependency_name == "xtask")
                || (name == "xtask" && dependency_name.starts_with("bevy"))
                || (name == "low-kingdoms" && dependency_name != "low-kingdoms-client")
            {
                return Err(Failure::new(format!(
                    "Invalid package boundary: {name} depends on {dependency_name}"
                )));
            }
        }
    }
    Ok(())
}

fn check_documents(project: &Project) -> Result<()> {
    for relative in [
        "AGENTS.md",
        "README.md",
        "STYLE_GUIDE.md",
        "THIRD_PARTY_NOTICES.md",
        "crates/client/AGENTS.md",
        "assets/README.md",
        "docs/architecture.md",
        "docs/development.md",
        "docs/testing.md",
        "docs/decisions/0001-scaffold.md",
        "docs/references/bevy.md",
        "docs/plans/scaffold.md",
    ] {
        let path = project.root.join(relative);
        if !path.is_file() {
            return Err(Failure::new(format!(
                "Missing required document {relative}"
            )));
        }
    }
    for name in ["bevy-change", "rust-quality", "verify-native-web"] {
        let path = project.root.join(format!(".agents/skills/{name}/SKILL.md"));
        let source = fs::read_to_string(&path)
            .map_err(|error| Failure::new(format!("Cannot read {}: {error}", path.display())))?;
        skill_metadata(&source, name)?;
    }
    Ok(())
}

fn check_browser_pins(project: &Project) -> Result<()> {
    let package_path = project.root.join("tools/web-tests/package.json");
    let package_source = fs::read_to_string(&package_path).map_err(|error| {
        Failure::new(format!("Cannot read {}: {error}", package_path.display()))
    })?;
    let package: serde_json::Value = serde_json::from_str(&package_source)
        .map_err(|error| Failure::new(format!("Invalid package.json: {error}")))?;
    for (name, tool) in [
        ("@playwright/test", "playwright"),
        ("http-server", "http-server"),
        ("prettier", "prettier"),
    ] {
        if package["devDependencies"][name].as_str() != Some(project.version(tool)?) {
            return Err(Failure::new(format!(
                "package.json must pin {name} to tools/tool-versions.toml"
            )));
        }
    }
    Ok(())
}

fn skill_metadata(source: &str, expected_name: &str) -> Result<()> {
    let mut lines = source.lines();
    if lines.next() != Some("---") {
        return Err(Failure::new(format!(
            "{expected_name}: missing YAML front matter"
        )));
    }
    let mut name = None;
    let mut description = None;
    let mut closed = false;
    for line in lines.by_ref() {
        if line == "---" {
            closed = true;
            break;
        }
        if let Some(value) = line.strip_prefix("name:") {
            name = Some(value.trim().trim_matches(['\'', '"']));
        }
        if let Some(value) = line.strip_prefix("description:") {
            description = Some(value.trim().trim_matches(['\'', '"']));
        }
    }
    if !closed || name != Some(expected_name) || description.is_none_or(str::is_empty) {
        return Err(Failure::new(format!(
            "{expected_name}: front matter needs matching name and nonempty one-line description"
        )));
    }
    if !lines.any(|line| !line.trim().is_empty()) {
        return Err(Failure::new(format!(
            "{expected_name}: skill procedure is empty"
        )));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn skill_header_requires_correct_name_and_procedure() {
        let valid = "---\nname: rust-quality\ndescription: Check Rust changes.\n---\nRun cargo xtask check.\n";
        assert!(skill_metadata(valid, "rust-quality").is_ok());
        assert!(skill_metadata(valid, "bevy-change").is_err());
        assert!(skill_metadata("---\nname: rust-quality\n---\nDo checks", "rust-quality").is_err());
        assert!(
            skill_metadata(
                "---\nname: rust-quality\ndescription: Check\n---",
                "rust-quality"
            )
            .is_err()
        );
    }
}
