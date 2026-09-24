use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::screen::{self, Target};

/// Разбор задачи на шаги. Один и тот же план ведёт и режим подсказок, и режим «сделай за меня».
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Plan {
    pub intro: String,
    pub steps: Vec<Step>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Step {
    pub hint: String,
    pub target: Target,
    pub action: Action,
    pub expect: Expect,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Action {
    Click,
    Type { text: String },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Expect {
    Appeared { target: Target },
    Disappeared { target: Target },
    TextContains { target: Target, text: String },
}

impl Expect {
    pub fn target(&self) -> &Target {
        match self {
            Self::Appeared { target }
            | Self::Disappeared { target }
            | Self::TextContains { target, .. } => target,
        }
    }
}

/// Делает шаг за пользователя — режим «покажи сам».
pub fn perform(step: &Step) -> Result<()> {
    match &step.action {
        Action::Click => screen::click(&step.target),
        Action::Type { text } => screen::type_text(&step.target, text),
    }
}

/// Шаг сделан — неважно, агентом или человеком. Проверка одна на оба режима.
pub fn is_done(expect: &Expect) -> Result<bool> {
    match expect {
        Expect::Appeared { target } => screen::exists(target),
        Expect::Disappeared { target } => Ok(!screen::exists(target)?),
        Expect::TextContains { target, text } => Ok(contains(&screen::text_of(target)?, text)),
    }
}

fn contains(haystack: &str, needle: &str) -> bool {
    haystack
        .to_lowercase()
        .contains(&needle.trim().to_lowercase())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compares_text_ignoring_case_and_padding() {
        assert!(contains("Заметка.txt", "  ЗАМЕТКА  "));
        assert!(!contains("Заметка.txt", "смета"));
    }

    #[test]
    fn reads_plan_written_by_the_model() {
        let plan: Plan = serde_json::from_str(
            r#"{
                "intro": "Сохраним заметку",
                "steps": [
                    {
                        "hint": "Нажмите меню Файл",
                        "target": { "name": "Файл", "control_type": "menu_item" },
                        "action": { "kind": "click" },
                        "expect": {
                            "kind": "appeared",
                            "target": { "name": "Сохранить как" }
                        }
                    },
                    {
                        "hint": "Впишите имя",
                        "target": { "name": "Имя файла", "control_type": "edit" },
                        "action": { "kind": "type", "text": "заметка.txt" },
                        "expect": {
                            "kind": "text_contains",
                            "target": { "name": "Имя файла", "control_type": "edit" },
                            "text": "заметка"
                        }
                    }
                ]
            }"#,
        )
        .unwrap();

        assert_eq!(plan.steps.len(), 2);
        assert!(matches!(plan.steps[0].action, Action::Click));
        assert!(matches!(plan.steps[1].expect, Expect::TextContains { .. }));
    }
}
