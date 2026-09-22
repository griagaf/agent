use anyhow::{Result, bail};
use serde::Serialize;
use serde_json::{Value, json};

use crate::llm::ToolSpec;
use crate::tools::Tool;

const DEFAULT_LIMIT: u64 = 5;
const MAX_LIMIT: u64 = 20;

pub struct FindElement;

pub(super) struct Query {
    pub(super) name: String,
    pub(super) control_type: Option<String>,
    pub(super) window: Option<String>,
    pub(super) limit: usize,
}

#[derive(Serialize)]
pub(super) struct Match {
    #[serde(skip)]
    pub(super) score: u8,
    pub(super) name: String,
    pub(super) control_type: String,
    pub(super) enabled: bool,
    pub(super) rect: Rect,
}

// Физические пиксели экрана; в логические их пересчитывает окно, когда рисует указатель.
#[derive(Serialize)]
pub(super) struct Rect {
    pub(super) left: i32,
    pub(super) top: i32,
    pub(super) right: i32,
    pub(super) bottom: i32,
}

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
        let matches = find(&parse_query(input)?)?;

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

#[cfg(windows)]
fn find(query: &Query) -> Result<Vec<Match>> {
    super::windows::find(query)
}

#[cfg(not(windows))]
fn find(_query: &Query) -> Result<Vec<Match>> {
    bail!("поиск элементов на экране пока сделан только для Windows");
}

fn parse_query(input: &Value) -> Result<Query> {
    let Some(name) = optional_text(input, "name") else {
        bail!("не указано, какой элемент искать");
    };

    Ok(Query {
        name,
        control_type: optional_text(input, "control_type"),
        window: optional_text(input, "window"),
        limit: input["max_results"]
            .as_u64()
            .unwrap_or(DEFAULT_LIMIT)
            .clamp(1, MAX_LIMIT) as usize,
    })
}

fn optional_text(input: &Value, key: &str) -> Option<String> {
    input[key]
        .as_str()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
}
