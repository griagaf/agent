use anyhow::{Result, bail};
use serde::Deserialize;
use serde_json::{Value, json};

use crate::llm::ToolSpec;
use crate::scenario::Plan;
use crate::tools::Tool;

const MAX_STEPS: usize = 12;

pub struct ProposePlan;

impl Tool for ProposePlan {
    fn spec(&self) -> ToolSpec {
        ToolSpec {
            name: "propose_plan".to_string(),
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
                                        Поля те же, что у find_element.",
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
                                    "description": "Признак, что шаг выполнен. \
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
        check(&plan)?;

        Ok(json!({
            "accepted": plan.steps.len(),
            "plan": plan,
            "next": "План принят. Теперь коротко скажи пользователю, что будете делать."
        })
        .to_string())
    }
}

fn check(plan: &Plan) -> Result<()> {
    if plan.steps.is_empty() {
        bail!("в плане нет ни одного шага");
    }
    if plan.steps.len() > MAX_STEPS {
        bail!("шагов больше {MAX_STEPS}: разбей задачу на части покороче");
    }

    for (index, step) in plan.steps.iter().enumerate() {
        let number = index + 1;
        if step.hint.trim().is_empty() {
            bail!("у шага {number} пустая подсказка");
        }
        if step.target.name.trim().is_empty() {
            bail!("у шага {number} не указан элемент");
        }
    }

    Ok(())
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
            "target": { "name": "Файл", "control_type": "menu_item" },
            "action": { "kind": "click" },
            "expect": { "kind": "appeared", "target": { "name": "Сохранить как" } }
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
    fn refuses_a_step_without_a_hint() {
        let mut step = one_step();
        step["hint"] = json!("   ");

        let error = ProposePlan.call(&plan_with(json!([step]))).unwrap_err();
        assert!(error.to_string().contains("подсказка"), "{error}");
    }
}
