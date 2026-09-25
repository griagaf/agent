use serde::{Deserialize, Serialize};

/// Как найти элемент: подпись, которую видит человек, плюс необязательные уточнения.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Target {
    pub name: String,
    #[serde(default)]
    pub control_type: Option<String>,
    #[serde(default)]
    pub window: Option<String>,
}

impl Target {
    /// Само окно: искать его внутри другого окна не нужно и негде.
    pub fn is_window(&self) -> bool {
        self.control_type
            .as_deref()
            .is_some_and(|kind| normalize(kind) == "window")
    }
}

pub(super) fn normalize(control_type: &str) -> String {
    control_type
        .trim()
        .to_lowercase()
        .replace(['_', '-', ' '], "")
}

#[derive(Debug, Clone, Serialize)]
pub struct Match {
    #[serde(skip)]
    pub score: u8,
    pub name: String,
    pub control_type: String,
    pub enabled: bool,
    pub rect: Rect,
}

/// Физические пиксели экрана; в логические их пересчитывает окно, когда рисует указатель.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Rect {
    pub left: i32,
    pub top: i32,
    pub right: i32,
    pub bottom: i32,
}
