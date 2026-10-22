//! Команды разработки для Rust-части проекта.
//!
//! Запускаются из любого места репозитория:
//!
//! ```text
//! cargo xtask check               форматирование, clippy, тесты, документация, границы ядра
//! cargo xtask coverage            тесты с порогом покрытия (нужен cargo-llvm-cov)
//! cargo xtask commits [диапазон]  формат коммитов, по умолчанию origin/main..HEAD
//! ```

mod boundaries;
mod commits;

use std::env;
use std::process::{Command, ExitCode};

/// Минимальное покрытие строк, функций и регионов, в процентах.
const COVERAGE_MIN: &str = "80";

/// Код запуска приложения проверяется запуском, а не модульными тестами.
const COVERAGE_IGNORE: &str = r"app[/\\]src-tauri[/\\]src[/\\](main|lib)\.rs$";

/// Куда `coverage` пишет отчёт в формате lcov.
const COVERAGE_LCOV: &str = "target/lcov.info";

const USAGE: &str = "Использование: cargo xtask <команда>

Команды:
  check               форматирование, clippy, тесты, документация, границы ядра
  coverage            тесты с порогом покрытия (нужен cargo-llvm-cov)
  commits [диапазон]  формат коммитов, по умолчанию origin/main..HEAD";

fn main() -> ExitCode {
    let args: Vec<String> = env::args().skip(1).collect();
    let result = match args.first().map(String::as_str) {
        Some("check") => check(),
        Some("coverage") => coverage(),
        Some("commits") => commits::check(args.get(1).map_or("origin/main..HEAD", String::as_str)),
        _ => Err(USAGE.to_owned()),
    };
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => {
            eprintln!("{message}");
            ExitCode::FAILURE
        }
    }
}

fn check() -> Result<(), String> {
    cargo(&["fmt", "--all", "--check"])?;
    cargo(&[
        "clippy",
        "--workspace",
        "--all-targets",
        "--",
        "-D",
        "warnings",
    ])?;
    cargo(&["test", "--workspace"])?;
    cargo_with_env(
        &["doc", "--workspace", "--no-deps"],
        &[("RUSTDOCFLAGS", "-D warnings")],
    )?;
    boundaries::check(&cargo_path())
}

fn coverage() -> Result<(), String> {
    cargo(&[
        "llvm-cov",
        "--workspace",
        "--exclude",
        "xtask",
        "--ignore-filename-regex",
        COVERAGE_IGNORE,
        "--fail-under-lines",
        COVERAGE_MIN,
        "--fail-under-functions",
        COVERAGE_MIN,
        "--fail-under-regions",
        COVERAGE_MIN,
    ])?;
    cargo(&[
        "llvm-cov",
        "report",
        "--ignore-filename-regex",
        COVERAGE_IGNORE,
        "--lcov",
        "--output-path",
        COVERAGE_LCOV,
    ])
}

fn cargo_path() -> String {
    env::var("CARGO").unwrap_or_else(|_| "cargo".to_owned())
}

fn cargo(args: &[&str]) -> Result<(), String> {
    cargo_with_env(args, &[])
}

fn cargo_with_env(args: &[&str], env: &[(&str, &str)]) -> Result<(), String> {
    let command = format!("cargo {}", args.join(" "));
    eprintln!("> {command}");
    let status = Command::new(cargo_path())
        .args(args)
        .envs(env.iter().copied())
        .status()
        .map_err(|err| format!("не удалось запустить {command}: {err}"))?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("{command} завершился с ошибкой"))
    }
}
