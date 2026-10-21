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
| корень | `cargo xtask check` | Форматирование, clippy и тесты Rust |
| корень | `cargo xtask coverage` | Покрытие Rust |

Порог покрытия 80%. Для покрытия Rust нужен [cargo-llvm-cov](https://github.com/taiki-e/cargo-llvm-cov): `cargo install cargo-llvm-cov`.

## Структура

```
Cargo.toml     Cargo workspace: общие версии зависимостей и линты
crates/        ядро и адаптеры (mentor-core, -engine, -llm, -platform, -storage, -testkit)
app/           интерфейс на Vue и TypeScript
app/src-tauri/ приложение Tauri (крейт mentor-app)
docs/          документация
xtask/         команды разработки для cargo xtask
```

Как устроен код и какие правила связывают пакеты, описано в [docs/architecture.md](docs/architecture.md).
