use agent::screen::Rect;
use anyhow::{Result, bail};
use log::error;
use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager, Monitor, WebviewUrl, WebviewWindowBuilder};

use crate::error::Error;

const LABEL: &str = "pointer";
const CHANNEL: &str = "pointer:target";
const LEVEL_CHANNEL: &str = "pointer:level";

// Ступени подсказки: сначала мягкая подсветка области, потом точная стрелка.
#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "snake_case")]
enum Level {
    Region,
    Pointer,
}

// Место стрелки внутри окна-оверлея, в логических пикселях CSS.
#[derive(Debug, Clone, Serialize)]
struct Target {
    left: f64,
    top: f64,
    width: f64,
    height: f64,
    level: Level,
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
            level: Level::Region,
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
pub fn hide_pointer(app: AppHandle) -> Result<(), Error> {
    hide(&app).map_err(|e| {
        error!("[hide_pointer] {e:#}");
        Error::PointerFailed
    })
}

pub(crate) fn point_at(app: &AppHandle, rect: Rect) -> Result<()> {
    let Some(window) = app.get_webview_window(LABEL) else {
        bail!("окно '{LABEL}' не создано");
    };

    let (x, y) = center(rect);
    let Some(monitor) = app.monitor_from_point(x, y)? else {
        bail!("для точки ({x}, {y}) нет монитора");
    };

    // Порядок важен: сначала накрываем монитор, потом показываем — иначе видно рывок.
    window.set_position(*monitor.position())?;
    window.set_size(*monitor.size())?;
    window.emit_to(LABEL, CHANNEL, Target::new(&monitor, rect))?;
    window.show()?;

    Ok(())
}

#[tauri::command]
pub fn raise_pointer(app: AppHandle) -> Result<(), Error> {
    raise(&app).map_err(|e| {
        error!("[raise_pointer] {e:#}");
        Error::PointerFailed
    })
}

fn raise(app: &AppHandle) -> Result<()> {
    let Some(window) = app.get_webview_window(LABEL) else {
        bail!("окно '{LABEL}' не создано");
    };

    window.emit_to(LABEL, LEVEL_CHANNEL, Level::Pointer)?;

    Ok(())
}

fn center(rect: Rect) -> (f64, f64) {
    (
        f64::from(rect.left + rect.right) / 2.0,
        f64::from(rect.top + rect.bottom) / 2.0,
    )
}

fn hide(app: &AppHandle) -> Result<()> {
    let Some(window) = app.get_webview_window(LABEL) else {
        bail!("окно '{LABEL}' не создано");
    };

    window.hide()?;

    Ok(())
}
