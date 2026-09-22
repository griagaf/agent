use anyhow::{Result, bail};
use log::error;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, Manager, Monitor, WebviewUrl, WebviewWindowBuilder};

use crate::error::Error;

const LABEL: &str = "pointer";
const CHANNEL: &str = "pointer:target";

/// Прямоугольник элемента в физических пикселях — в них же отдаёт координаты UI Automation.
#[derive(Debug, Clone, Copy, Deserialize)]
pub struct Rect {
    pub left: i32,
    pub top: i32,
    pub right: i32,
    pub bottom: i32,
}

impl Rect {
    fn center_x(&self) -> f64 {
        f64::from(self.left + self.right) / 2.0
    }

    fn center_y(&self) -> f64 {
        f64::from(self.top + self.bottom) / 2.0
    }
}

// Место стрелки внутри окна-оверлея, в логических пикселях CSS.
#[derive(Debug, Clone, Serialize)]
struct Target {
    left: f64,
    top: f64,
    width: f64,
    height: f64,
}

impl Target {
    fn new(monitor: &Monitor, rect: Rect) -> Self {
        let scale = monitor.scale_factor();
        let origin = monitor.position();

        Self {
            left: f64::from(rect.left - origin.x) / scale,
            top: f64::from(rect.top - origin.y) / scale,
            width: f64::from(rect.right - rect.left) / scale,
            height: f64::from(rect.bottom - rect.top) / scale,
        }
    }
}

pub fn create(app: &AppHandle) -> tauri::Result<()> {
    let window = WebviewWindowBuilder::new(app, LABEL, WebviewUrl::App("pointer.html".into()))
        .title("Указатель")
        .transparent(true)
        .decorations(false)
        .always_on_top(true)
        .skip_taskbar(true)
        .shadow(false)
        .resizable(false)
        .focused(false)
        .visible(false)
        .build()?;

    // Под оверлеем работает пользователь: клики и наведение должны проходить насквозь.
    window.set_ignore_cursor_events(true)
}

#[tauri::command]
pub fn show_pointer(app: AppHandle, target: Rect) -> Result<(), Error> {
    point_at(&app, target).map_err(|e| {
        error!("[show_pointer] {e:#}");
        Error::PointerFailed
    })
}

#[tauri::command]
pub fn hide_pointer(app: AppHandle) -> Result<(), Error> {
    hide(&app).map_err(|e| {
        error!("[hide_pointer] {e:#}");
        Error::PointerFailed
    })
}

fn point_at(app: &AppHandle, rect: Rect) -> Result<()> {
    let Some(window) = app.get_webview_window(LABEL) else {
        bail!("окно '{LABEL}' не создано");
    };

    let Some(monitor) = app.monitor_from_point(rect.center_x(), rect.center_y())? else {
        bail!(
            "для точки ({}, {}) нет монитора",
            rect.center_x(),
            rect.center_y()
        );
    };

    // Порядок важен: сначала накрываем монитор, потом показываем — иначе видно рывок.
    window.set_position(*monitor.position())?;
    window.set_size(*monitor.size())?;
    window.emit_to(LABEL, CHANNEL, Target::new(&monitor, rect))?;
    window.show()?;

    Ok(())
}

fn hide(app: &AppHandle) -> Result<()> {
    let Some(window) = app.get_webview_window(LABEL) else {
        bail!("окно '{LABEL}' не создано");
    };

    window.hide()?;

    Ok(())
}
