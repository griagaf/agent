use anyhow::{Result, bail};
use serde_json::{Value, json};

use crate::llm::ToolSpec;
use crate::screen::{self, Target};
use crate::tools::Tool;

const DEFAULT_LIMIT: u64 = 5;
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
                        "description": "Подпись элемента или её часть, как она видна \
                            на экране: 'Сохранить', 'Файл', 'Отмена'."
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
        let matches = screen::find(&parse_target(input)?, parse_limit(input))?;

        if matches.is_empty() {
            return Ok(json!({
                "found": 0,
                "results": [],
                "hint": "Такого элемента сейчас на экране нет. Возможно, нужное окно \
                    закрыто или свёрнуто, либо подпись отличается — стоит переспросить \
                    пользователя, что он видит."
            })
            .to_string());
        }

        Ok(json!({ "found": matches.len(), "results": matches }).to_string())
    }
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
