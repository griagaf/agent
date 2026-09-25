use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use crate::llm::ToolSpec;
use crate::scenario::Plan;
use crate::tools::Tool;

use super::check;

pub const NAME: &str = "propose_plan";

/// Вывод инструмента: его читает и модель, и окно, которое поведёт пользователя по шагам.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Accepted {
    pub accepted: usize,
    pub plan: Plan,
    pub next: String,
}

pub struct ProposePlan;

impl Tool for ProposePlan {
    fn spec(&self) -> ToolSpec {
        ToolSpec {
            name: NAME.to_string(),
            description: "Разбивает задачу на пошаговый план, по которому агент поведёт \
                пользователя: на каждом шаге он показывает стрелкой нужный элемент, \
                говорит подсказку и ждёт, пока человек сделает это сам. Вызывай, когда \
                понял, что нужно сделать и где это лежит на экране. Один шаг — одно \
                действие; подсказку пиши так, как объяснил бы человеку, который \
                компьютер видит впервые."
                .to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "intro": {
                        "type": "string",
                        "description": "Одна фраза о том, что сейчас будем делать."
                    },
                    "steps": {
                        "type": "array",
                        "description": "Шаги по порядку, не больше 12.",
                        "items": {
                            "type": "object",
                            "properties": {
                                "hint": {
                                    "type": "string",
                                    "description": "Что сказать человеку: 'Нажмите меню \
                                        Файл в левом верхнем углу'."
                                },
                                "target": {
                                    "type": "object",
                                    "description": "Элемент, на который показать стрелкой. \
                                        Поля те же, что у find_element, window обязателен. \
                                        Подписи бери из ответов find_element — выдуманные \
                                        на экране не найдутся, и шаг зависнет.",
                                    "properties": {
                                        "name": { "type": "string" },
                                        "control_type": { "type": "string" },
                                        "window": { "type": "string" }
                                    },
                                    "required": ["name"]
                                },
                                "action": {
                                    "type": "object",
                                    "description": "Что делают с элементом: \
                                        {'kind':'click'} или {'kind':'type','text':'...'}.",
                                    "properties": {
                                        "kind": { "enum": ["click", "type"] },
                                        "text": { "type": "string" }
                                    },
                                    "required": ["kind"]
                                },
                                "expect": {
                                    "type": "object",
                                    "description": "Признак, что шаг выполнен — то, чего \
                                        до шага на экране не было. Кнопка, которая видна \
                                        и так, проверкой быть не может: шаг засчитается \
                                        сразу, и человек его не заметит. В подписи для \
                                        text_contains оставляй только устойчивую часть, \
                                        без текущего значения: 'Отображать как', а не \
                                        'Отображать как 0'. \
                                        {'kind':'appeared','target':{...}} — элемент \
                                        появился; {'kind':'disappeared','target':{...}} — \
                                        пропал; {'kind':'text_contains','target':{...},\
                                        'text':'...'} — в поле нужный текст.",
                                    "properties": {
                                        "kind": {
                                            "enum": ["appeared", "disappeared", "text_contains"]
                                        },
                                        "target": { "type": "object" },
                                        "text": { "type": "string" }
                                    },
                                    "required": ["kind", "target"]
                                }
                            },
                            "required": ["hint", "target", "action", "expect"]
                        }
                    }
                },
                "required": ["intro", "steps"]
            }),
        }
    }

    fn call(&self, input: &Value) -> Result<String> {
        let plan = Plan::deserialize(input)?;
        check::plan(&plan)?;

        let accepted = Accepted {
            accepted: plan.steps.len(),
            plan,
            next: "План принят. Теперь коротко скажи пользователю, что будете делать.".to_string(),
        };

        Ok(serde_json::to_string(&accepted)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn plan_with(steps: Value) -> Value {
        json!({ "intro": "Сохраним заметку", "steps": steps })
    }

    fn one_step() -> Value {
        json!({
            "hint": "Нажмите меню Файл",
            "target": { "name": "Файл", "control_type": "menu_item", "window": "Блокнот" },
            "action": { "kind": "click" },
            "expect": {
                "kind": "appeared",
                "target": { "name": "Сохранить как", "window": "Блокнот" }
            }
        })
    }

    #[test]
    fn accepts_a_plan_and_returns_it_back() {
        let output = ProposePlan.call(&plan_with(json!([one_step()]))).unwrap();
        let parsed: Value = serde_json::from_str(&output).unwrap();

        assert_eq!(parsed["accepted"], 1);
        assert_eq!(parsed["plan"]["steps"][0]["target"]["name"], "Файл");
    }

    #[test]
    fn refuses_an_empty_plan() {
        let error = ProposePlan.call(&plan_with(json!([]))).unwrap_err();
        assert!(error.to_string().contains("ни одного шага"), "{error}");
    }

    #[test]
    fn refuses_a_check_that_repeats_the_step_itself() {
        let mut step = one_step();
        step["expect"]["target"]["name"] = json!("файл");

        let error = ProposePlan.call(&plan_with(json!([step]))).unwrap_err();
        assert!(error.to_string().contains("того же элемента"), "{error}");
    }

    #[test]
    fn allows_a_window_as_the_target_itself() {
        let mut step = one_step();
        step["expect"]["target"] = json!({ "name": "Калькулятор", "control_type": "window" });

        let output = ProposePlan.call(&plan_with(json!([step]))).unwrap();
        assert!(output.contains("accepted"), "{output}");
    }

    #[test]
    fn refuses_a_check_without_a_window() {
        let mut step = one_step();
        step["expect"]["target"]
            .as_object_mut()
            .unwrap()
            .remove("window");

        let error = ProposePlan.call(&plan_with(json!([step]))).unwrap_err();
        assert!(error.to_string().contains("у признака"), "{error}");
    }

    #[test]
    fn refuses_two_steps_with_the_same_check() {
        let plan = plan_with(json!([one_step(), one_step()]));

        let error = ProposePlan.call(&plan).unwrap_err();
        assert!(
            error.to_string().contains("один и тот же признак"),
            "{error}"
        );
    }

    #[test]
    fn refuses_a_step_without_a_window() {
        let mut step = one_step();
        step["target"].as_object_mut().unwrap().remove("window");

        let error = ProposePlan.call(&plan_with(json!([step]))).unwrap_err();
        assert!(error.to_string().contains("не указано окно"), "{error}");
    }

    #[test]
    fn refuses_a_step_without_a_hint() {
        let mut step = one_step();
        step["hint"] = json!("   ");

        let error = ProposePlan.call(&plan_with(json!([step]))).unwrap_err();
        assert!(error.to_string().contains("подсказка"), "{error}");
    }
}
