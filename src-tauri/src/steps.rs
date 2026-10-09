use agent::scenario::{self, Expect, Step};
use agent::screen;
use anyhow::{Context, Result};
use log::{error, info, warn};
use tauri::AppHandle;

use crate::error::Error;
use crate::overlay;

#[tauri::command]
pub async fn show_step(app: AppHandle, step: Step) -> Result<(), Error> {
    info!("[show_step] {}", step.target.name);

    let target = step.target.clone();
    let found = blocking(move || screen::find(&target, 1))
        .await
        .map_err(|reason| {
            error!("[show_step] {reason:#}");
            Error::StepFailed
        })?;

    let Some(first) = found.first() else {
        warn!("[show_step] элемента {} на экране нет", step.target.name);
        return Err(Error::ElementMissing);
    };

    overlay::point_at(&app, first.rect).map_err(|reason| {
        error!("[show_step] {reason:#}");
        Error::PointerFailed
    })
}

#[tauri::command]
pub async fn check_step(expect: Expect) -> Result<bool, Error> {
    let done = blocking(move || scenario::is_done(&expect))
        .await
        .map_err(|reason| {
            error!("[check_step] {reason:#}");
            Error::StepFailed
        })?;

    if done {
        info!("[check_step] шаг подтверждён");
    }

    Ok(done)
}

#[tauri::command]
pub async fn perform_step(step: Step) -> Result<(), Error> {
    info!("[perform_step] {}", step.target.name);

    blocking(move || scenario::perform(&step))
        .await
        .map_err(|reason| {
            error!("[perform_step] {reason:#}");
            Error::StepFailed
        })
}

// UI Automation блокирует поток надолго, а COM требует инициализации на своём.
async fn blocking<T, F>(job: F) -> Result<T>
where
    F: FnOnce() -> Result<T> + Send + 'static,
    T: Send + 'static,
{
    tauri::async_runtime::spawn_blocking(job)
        .await
        .context("фоновый поток сорвался")?
}
