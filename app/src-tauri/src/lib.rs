//! Приложение Tauri: окна, команды и трей.

/// Собирает и запускает приложение.
pub fn run() {
    tauri::Builder::default()
        .run(tauri::generate_context!())
        .expect("не удалось запустить приложение");
}
