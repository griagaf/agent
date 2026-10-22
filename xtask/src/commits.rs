//! Формат заголовков коммитов: `тип(область): описание`, Conventional Commits
//! с закрытыми списками типов и областей проекта.

use std::process::Command;

const TYPES: &[&str] = &[
    "feat", "fix", "perf", "refactor", "test", "docs", "ci", "build", "chore", "style", "revert",
];

const SCOPES: &[&str] = &[
    "core", "engine", "llm", "platform", "storage", "testkit", "app", "ui", "ci", "repo",
    "release", "docs",
];

const MAX_HEADER_CHARS: usize = 72;

/// Проверяет заголовки всех коммитов диапазона, кроме merge-коммитов.
pub fn check(range: &str) -> Result<(), String> {
    eprintln!("> git log --no-merges {range}");
    let output = Command::new("git")
        .args(["log", "--no-merges", "--format=%h %s", range])
        .output()
        .map_err(|err| format!("не удалось запустить git: {err}"))?;
    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).into_owned());
    }

    let errors: Vec<String> = String::from_utf8_lossy(&output.stdout)
        .lines()
        .filter_map(|line| {
            let (hash, header) = line.split_once(' ').unwrap_or((line, ""));
            validate(header)
                .err()
                .map(|reason| format!("{hash} {header}\n    {reason}"))
        })
        .collect();

    if errors.is_empty() {
        Ok(())
    } else {
        Err(format!(
            "Коммиты не по формату «тип(область): описание»:\n{}",
            errors.join("\n")
        ))
    }
}

/// Проверяет один заголовок коммита.
fn validate(header: &str) -> Result<(), String> {
    if header.chars().count() > MAX_HEADER_CHARS {
        return Err(format!("заголовок длиннее {MAX_HEADER_CHARS} символов"));
    }
    let (prefix, description) = header
        .split_once(": ")
        .ok_or("нет «тип(область): » в начале")?;
    let prefix = prefix.strip_suffix('!').unwrap_or(prefix);
    let (kind, scope) = prefix
        .strip_suffix(')')
        .and_then(|prefix| prefix.split_once('('))
        .ok_or("область обязательна: тип(область)")?;

    if !TYPES.contains(&kind) {
        return Err(format!("тип «{kind}» не из списка: {}", TYPES.join(", ")));
    }
    if !SCOPES.contains(&scope) {
        return Err(format!(
            "область «{scope}» не из списка: {}",
            SCOPES.join(", ")
        ));
    }
    if description.trim().is_empty() {
        return Err("пустое описание".to_owned());
    }
    if description.ends_with('.') {
        return Err("точка в конце описания".to_owned());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_project_headers() {
        for header in [
            "feat(core): контракт шага: наблюдение, план, условия и исход действия",
            "build(repo): cargo xtask для проверок Rust и покрытия",
            "test(ui): проверка фронта одной командой с порогом покрытия 80%",
            "feat(core)!: View получает номер версии",
        ] {
            assert_eq!(validate(header), Ok(()), "{header}");
        }
    }

    #[test]
    fn rejects_malformed_headers() {
        let cases = [
            ("fix stuff", "нет «тип(область): » в начале"),
            ("feat: без области", "область обязательна: тип(область)"),
            ("feat(core) без двоеточия", "нет «тип(область): » в начале"),
            ("feat(core): ", "пустое описание"),
            ("feat(core): описание с точкой.", "точка в конце описания"),
        ];
        for (header, reason) in cases {
            assert_eq!(validate(header), Err(reason.to_owned()), "{header}");
        }
    }

    #[test]
    fn rejects_unknown_type_and_scope() {
        let unknown_type = validate("feature(core): описание").unwrap_err();
        let unknown_scope = validate("feat(backend): описание").unwrap_err();

        assert!(unknown_type.starts_with("тип «feature»"), "{unknown_type}");
        assert!(
            unknown_scope.starts_with("область «backend»"),
            "{unknown_scope}"
        );
    }

    #[test]
    fn limits_header_length_in_characters() {
        let fits = format!("feat(core): {}", "я".repeat(MAX_HEADER_CHARS - 12));
        let too_long = format!("{fits}я");

        assert_eq!(validate(&fits), Ok(()));
        assert!(validate(&too_long).is_err());
    }
}
