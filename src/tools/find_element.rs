use std::collections::HashSet;

use anyhow::{Result, bail};
use serde_json::{Value, json};

use crate::llm::ToolSpec;
use crate::screen::{self, Target};
use crate::tools::Tool;

const DEFAULT_LIMIT: u64 = 5;
const NEARBY_LIMIT: usize = 12;
const NEARBY_SCAN: usize = 60;
const MAX_LIMIT: u64 = 20;

pub struct FindElement;

impl Tool for FindElement {
    fn spec(&self) -> ToolSpec {
        ToolSpec {
            name: "find_element".to_string(),
            description: "Находит на экране элемент управления — кнопку, пункт меню, поле \
                ввода — по подписи, которую видит человек, и возвращает его положение \
                в пикселях. Нужен, чтобы показать пользователю, куда нажать. Ищет только \
                среди того, что сейчас открыто на экране: если нужное окно закрыто, \
                сначала попроси пользователя его открыть."
                .to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "name": {
                        "type": "string",
                        "description": "Подпись элемента — то, что читает экранный \
                            диктор, а не то, что нарисовано: у кнопок калькулятора это \
                            'Пять' и 'Плюс', а не '5' и '+'. Не угадал — возьми подпись \
                            из поля nearby в ответе."
                    },
                    "control_type": {
                        "type": "string",
                        "description": "Необязательно: вид элемента. Допустимо button, \
                            menu, menu_item, edit, text, list, list_item, tab_item, \
                            check_box, radio_button, combo_box, hyperlink, tree_item, window."
                    },
                    "window": {
                        "type": "string",
                        "description": "Необязательно: часть заголовка окна, чтобы искать \
                            только в нём. Одинаковые подписи есть в разных программах."
                    },
                    "max_results": {
                        "type": "integer",
                        "description": "Сколько совпадений вернуть, по умолчанию 5."
                    }
                },
                "required": ["name"]
            }),
        }
    }

    fn call(&self, input: &Value) -> Result<String> {
        let target = parse_target(input)?;
        let matches = screen::find(&target, parse_limit(input))?;

        if matches.is_empty() {
            return Ok(nothing_found(&target).to_string());
        }

        Ok(json!({ "found": matches.len(), "results": matches }).to_string())
    }
}

// На промахе важнее не «нет», а «вот что есть»: так модель поправится за один шаг,
// а не станет перебирать подписи наугад.
fn nothing_found(target: &Target) -> Value {
    let Some(window) = &target.window else {
        return json!({
            "found": 0,
            "results": [],
            "open_windows": screen::windows().unwrap_or_default(),
            "hint": "По всему экрану ничего не нашлось. Возьми нужное окно из \
                open_windows и повтори поиск с полем window — так быстрее и точнее."
        });
    };

    json!({
        "found": 0,
        "results": [],
        "nearby": nearby_labels(target, window),
        "hint": "В этом окне такого элемента нет. Возьми подпись из поля nearby: \
            в Windows они бывают неожиданными. Если список пуст, окно закрыто — \
            попроси пользователя открыть его."
    })
}

fn nearby_labels(target: &Target, window: &str) -> Vec<String> {
    let anything = Target {
        name: String::new(),
        control_type: target.control_type.clone(),
        window: Some(window.to_string()),
    };

    // Порядок поиска ставит короткие подписи вперёд — это и есть простые кнопки.
    let mut seen = HashSet::new();

    screen::find(&anything, NEARBY_SCAN)
        .unwrap_or_default()
        .into_iter()
        .map(|found| found.name)
        .filter(|name| !name.trim().is_empty())
        .filter(|name| seen.insert(name.clone()))
        .take(NEARBY_LIMIT)
        .collect()
}

fn parse_target(input: &Value) -> Result<Target> {
    let Some(name) = optional_text(input, "name") else {
        bail!("не указано, какой элемент искать");
    };

    Ok(Target {
        name,
        control_type: optional_text(input, "control_type"),
        window: optional_text(input, "window"),
    })
}

fn parse_limit(input: &Value) -> usize {
    input["max_results"]
        .as_u64()
        .unwrap_or(DEFAULT_LIMIT)
        .clamp(1, MAX_LIMIT) as usize
}

fn optional_text(input: &Value, key: &str) -> Option<String> {
    input[key]
        .as_str()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
}
