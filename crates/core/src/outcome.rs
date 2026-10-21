//! Исход действия человека и порядок эскалации, когда код не справился.

use serde::{Deserialize, Serialize};

use crate::condition::Verdict;

/// Куда пришёлся щелчок относительно обведённой цели.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ClickPlace {
    /// По цели.
    OnTarget,
    /// Мимо цели.
    Elsewhere,
}

/// Исход действия, который определил код.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Outcome {
    /// Щелчок по цели, условие выполнено.
    Done,
    /// Щелчок мимо, но условие выполнено: действие засчитывается.
    DoneAnotherWay,
    /// Щелчок по цели, а условие не выполнилось.
    NoEffect,
    /// Щелчок мимо, и экран изменился: человек ушёл не туда.
    Slip,
    /// Щелчок мимо, на экране ничего не изменилось: подсказка ждёт дальше.
    Ignored,
}

/// Определяет исход действия по месту щелчка, проверке условия и тому,
/// изменился ли экран.
///
/// Возвращает `None`, если код судить не может и исход определяет модель.
pub fn outcome(place: ClickPlace, verdict: Verdict, screen_changed: bool) -> Option<Outcome> {
    match (place, verdict, screen_changed) {
        (ClickPlace::OnTarget, Verdict::Met, _) => Some(Outcome::Done),
        (ClickPlace::Elsewhere, Verdict::Met, _) => Some(Outcome::DoneAnotherWay),
        (ClickPlace::OnTarget, Verdict::NotYet, _) => Some(Outcome::NoEffect),
        (ClickPlace::Elsewhere, Verdict::NotYet, true) => Some(Outcome::Slip),
        (ClickPlace::Elsewhere, Verdict::NotYet, false) => Some(Outcome::Ignored),
        (_, Verdict::Unknown, _) => None,
    }
}

/// Что делать, когда код не справился сам. Варианты перечислены в порядке эскалации.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Escalation {
    /// Модель выбирает цель из списка кандидатов.
    ShortCall,
    /// Модель ищет цель по снимку экрана.
    VisionCall,
    /// Модель перестраивает план.
    Replan,
    /// Спросить человека.
    AskHuman,
}

#[cfg(test)]
mod tests {
    use super::*;
    use ClickPlace::{Elsewhere, OnTarget};
    use Verdict::{Met, NotYet, Unknown};

    #[test]
    fn met_condition_counts_wherever_the_click_was() {
        for changed in [true, false] {
            assert_eq!(outcome(OnTarget, Met, changed), Some(Outcome::Done));
            assert_eq!(
                outcome(Elsewhere, Met, changed),
                Some(Outcome::DoneAnotherWay)
            );
        }
    }

    #[test]
    fn click_on_target_without_effect() {
        for changed in [true, false] {
            assert_eq!(outcome(OnTarget, NotYet, changed), Some(Outcome::NoEffect));
        }
    }

    #[test]
    fn click_elsewhere_depends_on_screen_change() {
        assert_eq!(outcome(Elsewhere, NotYet, true), Some(Outcome::Slip));
        assert_eq!(outcome(Elsewhere, NotYet, false), Some(Outcome::Ignored));
    }

    #[test]
    fn unknown_verdict_is_left_to_the_model() {
        for place in [OnTarget, Elsewhere] {
            for changed in [true, false] {
                assert_eq!(outcome(place, Unknown, changed), None);
            }
        }
    }

    #[test]
    fn escalation_goes_from_cheap_to_human() {
        use Escalation::*;

        let mut order = [AskHuman, ShortCall, Replan, VisionCall];
        order.sort();

        assert_eq!(order, [ShortCall, VisionCall, Replan, AskHuman]);
    }
}
