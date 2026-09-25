use anyhow::{Result, bail};

use crate::scenario::{Expect, Plan, Step};
use crate::screen;

const MAX_STEPS: usize = 12;

pub(super) fn plan(plan: &Plan) -> Result<()> {
    if plan.steps.is_empty() {
        bail!("в плане нет ни одного шага");
    }
    if plan.steps.len() > MAX_STEPS {
        bail!("шагов больше {MAX_STEPS}: разбей задачу на части покороче");
    }

    for (index, step) in plan.steps.iter().enumerate() {
        check_step(step, index + 1)?;
    }

    if let Some(first) = plan.steps.first() {
        check_first_target(first)?;
    }

    for (index, pair) in plan.steps.windows(2).enumerate() {
        if pair[0].expect == pair[1].expect {
            bail!(
                "у шагов {} и {} один и тот же признак: второй подтвердить нечем, \
                 дай ему собственный",
                index + 1,
                index + 2
            );
        }
    }

    Ok(())
}

// Текст проверяют в поле, которое уже на экране. На первом шаге это проверяемо, и
// именно там модель чаще всего ошибается подписью.
fn check_first_target(step: &Step) -> Result<()> {
    let Expect::TextContains { target, .. } = &step.expect else {
        return Ok(());
    };

    if screen::exists(target).unwrap_or(false) {
        return Ok(());
    }

    bail!(
        "признак шага 1 смотрит на элемент '{}', которого на экране нет. Найди \
         его через find_element и возьми подпись оттуда, без текущего значения",
        target.name
    );
}

fn check_step(step: &Step, number: usize) -> Result<()> {
    if step.hint.trim().is_empty() {
        bail!("у шага {number} пустая подсказка");
    }
    if step.target.name.trim().is_empty() {
        bail!("у шага {number} не указан элемент");
    }
    if step.target.window.is_none() && !step.target.is_window() {
        bail!("у шага {number} не указано окно: без него поиск идёт секунды");
    }

    let signal = step.expect.target();
    if signal.window.is_none() && !signal.is_window() {
        bail!("у признака шага {number} не указано окно: его проверяют много раз в секунду");
    }

    let Expect::Appeared { target } = &step.expect else {
        return Ok(());
    };

    if same_label(&target.name, &step.target.name) {
        bail!(
            "шаг {number} проверяет появление того же элемента, по которому и жмёт. \
             Признаком должно быть то, чего до шага не было: открывшееся меню, \
             новое окно, изменившийся текст в поле"
        );
    }

    Ok(())
}

fn same_label(left: &str, right: &str) -> bool {
    left.trim().to_lowercase() == right.trim().to_lowercase()
}
