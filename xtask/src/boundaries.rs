//! Границы ядра: `mentor-core` не зависит от асинхронной среды, сети, ОС и Tauri.

use std::process::Command;

/// Пакет ядра, границы которого проверяются.
const CORE_PACKAGE: &str = "mentor-core";

/// Пакеты, которых не должно быть в дереве зависимостей ядра.
const FORBIDDEN: &[&str] = &[
    "tokio",
    "reqwest",
    "windows",
    "windows-sys",
    "objc2",
    "tauri",
];

/// Проверяет дерево зависимостей `mentor-core` на всех целевых платформах.
pub fn check(cargo: &str) -> Result<(), String> {
    eprintln!("> cargo tree --package {CORE_PACKAGE}");
    let output = Command::new(cargo)
        .args([
            "tree",
            "--package",
            CORE_PACKAGE,
            "--edges",
            "normal",
            "--target",
            "all",
            "--prefix",
            "none",
        ])
        .output()
        .map_err(|err| format!("не удалось запустить cargo tree: {err}"))?;
    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).into_owned());
    }

    let tree = String::from_utf8_lossy(&output.stdout);
    let found = forbidden_in(&tree);
    if found.is_empty() {
        Ok(())
    } else {
        Err(format!(
            "{CORE_PACKAGE} не должен зависеть от: {}",
            found.join(", ")
        ))
    }
}

/// Возвращает запрещённые пакеты из вывода `cargo tree --prefix none`.
fn forbidden_in(tree: &str) -> Vec<&str> {
    let mut found: Vec<&str> = tree
        .lines()
        .filter_map(|line| line.split_whitespace().next())
        .filter(|name| FORBIDDEN.contains(name))
        .collect();
    found.sort_unstable();
    found.dedup();
    found
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clean_tree_passes() {
        let tree = "mentor-core v0.0.0 (/repo/crates/core)\nserde v1.0.228\nserde_derive v1.0.228 (proc-macro)\n";

        assert!(forbidden_in(tree).is_empty());
    }

    #[test]
    fn forbidden_packages_are_reported_once() {
        let tree = "mentor-core v0.0.0\ntokio v1.48.0\nwindows-sys v0.61.2\ntokio v1.48.0 (*)\n";

        assert_eq!(forbidden_in(tree), ["tokio", "windows-sys"]);
    }

    #[test]
    fn similar_names_are_not_forbidden() {
        let tree = "tokio-util v0.7.0\nwindows-core v0.62.0\n";

        assert!(forbidden_in(tree).is_empty());
    }
}
