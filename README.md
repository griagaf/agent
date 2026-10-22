# Агент-наставник

Настольное приложение для людей, которым трудно даётся компьютер. Человек пишет или говорит, что ему нужно, а агент составляет план, обводит нужное место на экране и ведёт по шагам. Нажимает всегда сам человек.

## Запуск

Понадобятся [Rust](https://rustup.rs), [bun](https://bun.sh) и [системные зависимости Tauri](https://tauri.app/start/prerequisites/). Нужную версию Rust rustup поставит сам по `rust-toolchain.toml`.

```sh
cd app
bun install
bun run tauri dev
```

## Проверки

| Где | Команда | Что проверяет |
|---|---|---|
| `app/` | `bun run check` | Типы, тесты и покрытие фронта |
| корень | `cargo xtask check` | Форматирование, clippy, тесты, документацию и границы ядра |
| корень | `cargo xtask coverage` | Покрытие Rust, отчёт в `target/lcov.info` |
| корень | `cargo xtask commits` | Формат коммитов ветки относительно `origin/main` |
| корень | `cargo deny check` | Лицензии, уязвимости и источники зависимостей |

Порог покрытия 80%. Для покрытия Rust нужен [cargo-llvm-cov](https://github.com/taiki-e/cargo-llvm-cov), для проверки зависимостей [cargo-deny](https://github.com/EmbarkStudios/cargo-deny), для релиза [git-cliff](https://git-cliff.org): `cargo install cargo-llvm-cov cargo-deny git-cliff`.

Те же проверки GitHub Actions запускает на каждый пулл-реквест, на macOS и Windows.

## Релиз

Версия приложения задаётся в одном месте: `version` в разделе `[workspace.package]` корневого `Cargo.toml`. Пакеты и Tauri берут её оттуда.

1. От свежего `main` создать ветку `release/vX.Y.Z`.
2. `cargo xtask release X.Y.Z` обновит версию, `Cargo.lock` и `CHANGELOG.md`.
3. Закоммитить как `chore(release): vX.Y.Z`, открыть пулл-реквест и влить.
4. Поставить тег на `main`: `git tag -a vX.Y.Z -m vX.Y.Z` и `git push origin vX.Y.Z`.

По тегу GitHub Actions собирает установщик для Windows и создаёт черновик релиза с заметками из changelog. Версии вида `X.Y.Z-rc.N` публикуются как предварительные. Каждую ночь, если в `main` были изменения, тот же установщик собирается артефактом прогона.

## Структура

```
Cargo.toml     Cargo workspace: общие версии зависимостей и линты
crates/        ядро и адаптеры (mentor-core, -engine, -llm, -platform, -storage, -testkit)
app/           интерфейс на Vue и TypeScript
app/src-tauri/ приложение Tauri (крейт mentor-app)
docs/          документация
xtask/         команды разработки для cargo xtask
cliff.toml     шаблон changelog для git-cliff
deny.toml      правила cargo-deny
```

Как устроен код и какие правила связывают пакеты, описано в [docs/architecture.md](docs/architecture.md).
