//! Подготовка релиза: версия в `Cargo.toml` и `Cargo.lock`, запись в `CHANGELOG.md`.
//!
//! Версия приложения задана один раз, в `[workspace.package]` корневого `Cargo.toml`.
//! Остальные пакеты и Tauri берут её оттуда.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Манифест workspace, в котором задана версия.
const MANIFEST: &str = "Cargo.toml";

/// Раздел манифеста с общей для всех пакетов версией.
const VERSION_SECTION: &str = "[workspace.package]";

/// Файл, в который git-cliff пишет историю изменений.
const CHANGELOG: &str = "CHANGELOG.md";

/// Утилита, которая собирает changelog из заголовков коммитов.
const GIT_CLIFF: &str = "git-cliff";

/// Префикс ветки, в которой готовится релиз: `release/v1.2.3`.
const RELEASE_BRANCH_PREFIX: &str = "release/";

/// Префикс тега перед номером версии: `v1.2.3`.
const TAG_PREFIX: &str = "v";

/// Число частей номера версии: мажорная, минорная и патч.
const VERSION_PARTS: usize = 3;

/// Пометка выпуска-кандидата после дефиса: `1.0.0-rc.2`.
const CANDIDATE_PREFIX: &str = "rc.";

/// Обновляет версию и changelog. Коммит, пулл-реквест и тег делает разработчик.
pub fn prepare(version: &str, cargo: &str) -> Result<(), String> {
    check_version(version)?;
    let root = workspace_root();
    ensure_release_branch(&root)?;
    output(&root, GIT_CLIFF, &["--version"])
        .map_err(|_| format!("для релиза нужен {GIT_CLIFF}: cargo install {GIT_CLIFF}"))?;

    let manifest_path = root.join(MANIFEST);
    let manifest = fs::read_to_string(&manifest_path)
        .map_err(|err| format!("не удалось прочитать {MANIFEST}: {err}"))?;
    let updated = set_workspace_version(&manifest, version)?;

    let tag = format!("{TAG_PREFIX}{version}");
    write_release_files(&root, &manifest_path, &updated, cargo, &tag).map_err(|err| {
        format!(
            "{err}\nРелиз не подготовлен до конца. Рабочее дерево было чистым, откатить так:\n  \
             git reset --hard\n  \
             git clean -f {CHANGELOG}"
        )
    })?;

    eprintln!(
        "\nВерсия {version} подготовлена: {MANIFEST}, Cargo.lock, {CHANGELOG}.\n\
         Дальше:\n  \
         git add {MANIFEST} Cargo.lock {CHANGELOG}\n  \
         git commit -m \"chore(release): {tag}\"\n  \
         git push -u origin {RELEASE_BRANCH_PREFIX}{tag}\n\
         После слияния пулл-реквеста:\n  \
         git switch main\n  \
         git pull\n  \
         git tag -a {tag} -m {tag}\n  \
         git push origin {tag}"
    );
    Ok(())
}

/// Шаги, после которых рабочее дерево уже изменено.
fn write_release_files(
    root: &Path,
    manifest_path: &Path,
    manifest: &str,
    cargo: &str,
    tag: &str,
) -> Result<(), String> {
    fs::write(manifest_path, manifest)
        .map_err(|err| format!("не удалось записать {MANIFEST}: {err}"))?;
    run(root, "cargo", cargo, &["update", "--workspace"])?;
    run(
        root,
        GIT_CLIFF,
        GIT_CLIFF,
        &["--tag", tag, "--output", CHANGELOG],
    )
}

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("xtask лежит внутри workspace")
        .to_path_buf()
}

fn ensure_release_branch(root: &Path) -> Result<(), String> {
    let status = output(root, "git", &["status", "--porcelain"])?;
    if !status.trim().is_empty() {
        return Err("в рабочем дереве есть незакоммиченные изменения".to_owned());
    }
    let branch = output(root, "git", &["branch", "--show-current"])?;
    let branch = branch.trim();
    if !branch.starts_with(RELEASE_BRANCH_PREFIX) {
        return Err(format!(
            "релиз готовится в ветке {RELEASE_BRANCH_PREFIX}{TAG_PREFIX}X.Y.Z, а сейчас ветка «{branch}»"
        ));
    }
    Ok(())
}

/// Проверяет номер версии: `X.Y.Z` или выпуск-кандидат `X.Y.Z-rc.N`.
fn check_version(version: &str) -> Result<(), String> {
    let (core, candidate) = match version.split_once('-') {
        Some((core, candidate)) => (core, Some(candidate)),
        None => (version, None),
    };
    let parts: Vec<&str> = core.split('.').collect();
    let core_ok = parts.len() == VERSION_PARTS && parts.iter().all(|part| is_number(part));
    let candidate_ok = candidate.is_none_or(|candidate| {
        candidate
            .strip_prefix(CANDIDATE_PREFIX)
            .is_some_and(is_number)
    });
    if core_ok && candidate_ok {
        Ok(())
    } else {
        Err(format!(
            "версия «{version}» не похожа на X.Y.Z или X.Y.Z-{CANDIDATE_PREFIX}N"
        ))
    }
}

/// Число без ведущих нулей.
fn is_number(part: &str) -> bool {
    !part.is_empty()
        && part.bytes().all(|byte| byte.is_ascii_digit())
        && (part == "0" || !part.starts_with('0'))
}

/// Заменяет `version` в разделе [`VERSION_SECTION`], остальной текст не трогает.
fn set_workspace_version(manifest: &str, version: &str) -> Result<String, String> {
    let mut section = "";
    let mut replaced = false;
    let lines: Vec<String> = manifest
        .lines()
        .map(|line| {
            let trimmed = line.trim();
            if trimmed.starts_with('[') {
                section = trimmed;
            } else if section == VERSION_SECTION
                && !replaced
                && trimmed
                    .strip_prefix("version")
                    .is_some_and(|rest| rest.trim_start().starts_with('='))
            {
                replaced = true;
                return format!("version = \"{version}\"");
            }
            line.to_owned()
        })
        .collect();

    if replaced {
        Ok(lines.join("\n") + "\n")
    } else {
        Err(format!("в {MANIFEST} нет version в {VERSION_SECTION}"))
    }
}

/// Запускает программу в корне workspace. `name` — как показать её в журнале.
fn run(root: &Path, name: &str, program: &str, args: &[&str]) -> Result<(), String> {
    eprintln!("> {name} {}", args.join(" "));
    let status = Command::new(program)
        .args(args)
        .current_dir(root)
        .status()
        .map_err(|err| format!("не удалось запустить {name}: {err}"))?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("{name} завершился с ошибкой"))
    }
}

fn output(root: &Path, program: &str, args: &[&str]) -> Result<String, String> {
    let output = Command::new(program)
        .args(args)
        .current_dir(root)
        .output()
        .map_err(|err| format!("не удалось запустить {program}: {err}"))?;
    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).into_owned())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).into_owned())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Корневой манифест в том виде, в каком его меняет релиз.
    const WORKSPACE_MANIFEST: &str = include_str!("../testdata/workspace.toml");

    #[test]
    fn accepts_release_and_candidate_versions() {
        for version in ["0.0.1", "1.10.0", "0.9.0-rc.1", "2.0.0-rc.12"] {
            assert_eq!(check_version(version), Ok(()), "{version}");
        }
    }

    #[test]
    fn rejects_malformed_versions() {
        for version in [
            "",
            "1.0",
            "1.0.0.0",
            "v1.0.0",
            "01.0.0",
            "1.0.x",
            "1.0.0-beta.1",
            "1.0.0-rc.",
            "1.0.0-rc.01",
        ] {
            assert!(check_version(version).is_err(), "{version}");
        }
    }

    #[test]
    fn replaces_only_workspace_package_version() {
        let updated = set_workspace_version(WORKSPACE_MANIFEST, "0.1.0").unwrap();

        assert_eq!(
            updated,
            WORKSPACE_MANIFEST.replace(r#"version = "0.0.0""#, r#"version = "0.1.0""#)
        );
        assert!(updated.contains(r#"serde = { version = "1""#));
    }

    #[test]
    fn missing_workspace_version_is_an_error() {
        let manifest = WORKSPACE_MANIFEST.replace("version = \"0.0.0\"\n", "");

        assert!(set_workspace_version(&manifest, "0.1.0").is_err());
    }
}
