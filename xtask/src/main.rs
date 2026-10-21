//! Команды разработки для Rust-части проекта.
//!
//! Запускаются из любого места репозитория:
//!
//! ```text
//! cargo xtask check      форматирование, clippy и тесты
//! cargo xtask coverage   тесты с порогом покрытия (нужен cargo-llvm-cov)
//! ```

use std::env;
use std::process::{Command, ExitCode};

/// Минимальное покрытие строк, функций и регионов, в процентах.
const COVERAGE_MIN: &str = "80";

/// Код запуска приложения проверяется запуском, а не модульными тестами.
const COVERAGE_IGNORE: &str = r"app[/\\]src-tauri[/\\]src[/\\](main|lib)\.rs$";

const USAGE: &str = "Использование: cargo xtask <команда>

Команды:
  check      форматирование, clippy и тесты
  coverage   тесты с порогом покрытия (нужен cargo-llvm-cov)";

fn main() -> ExitCode {
    let steps: &[&[&str]] = match env::args().nth(1).as_deref() {
        Some("check") => &[
            &["fmt", "--all", "--check"],
            &[
                "clippy",
                "--workspace",
                "--all-targets",
                "--",
                "-D",
                "warnings",
            ],
            &["test", "--workspace"],
        ],
        Some("coverage") => &[&[
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
        ]],
        _ => {
            eprintln!("{USAGE}");
            return ExitCode::FAILURE;
        }
    };

    let cargo = env::var("CARGO").unwrap_or_else(|_| "cargo".to_owned());
    for args in steps {
        eprintln!("> cargo {}", args.join(" "));
        match Command::new(&cargo).args(*args).status() {
            Ok(status) if status.success() => {}
            Ok(_) => return ExitCode::FAILURE,
            Err(err) => {
                eprintln!("не удалось запустить cargo: {err}");
                return ExitCode::FAILURE;
            }
        }
    }
    ExitCode::SUCCESS
}
