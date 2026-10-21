// Без этого в релизной сборке на Windows рядом с окном открывается консоль.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
//! Точка входа приложения.

fn main() {
    mentor_app_lib::run()
}
