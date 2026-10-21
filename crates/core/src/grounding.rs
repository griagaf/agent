//! Поиск цели на экране: итог поиска и подпись найденной цели.

use serde::{Deserialize, Serialize};

use crate::observation::{AppId, ElementRef, Epoch, PxRect, Role};

/// Итог поиска цели.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Grounding {
    /// Цель найдена.
    Found {
        /// Элемент.
        element: ElementRef,
        /// Положение элемента.
        rect: PxRect,
        /// Каким способом найден.
        by: GroundedBy,
        /// Номер снимка, на котором искали.
        epoch: Epoch,
    },
    /// Подходят несколько элементов, выбор уходит модели.
    Ambiguous {
        /// Подходящие элементы.
        candidates: Vec<Candidate>,
        /// Номер снимка, на котором искали.
        epoch: Epoch,
    },
    /// Ничего не подошло.
    NotFound {
        /// Номер снимка, на котором искали.
        epoch: Epoch,
    },
}

/// Каким способом найдена цель. Способы перечислены от быстрого к медленному.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GroundedBy {
    /// По подписи, запомненной в прошлый раз.
    Memory,
    /// Точное совпадение имени и роли.
    ExactMatch,
    /// Оценка по имени, роли, области окна и соседям.
    Scored,
    /// Модель выбрала из списка кандидатов.
    ShortCall,
    /// Модель нашла по снимку экрана.
    VisionCall,
}

/// Элемент, похожий на цель.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Candidate {
    /// Элемент.
    pub element: ElementRef,
    /// Оценка сходства: чем больше, тем ближе к описанию цели.
    pub score: f32,
}

/// Подпись найденной цели, по которой её можно узнать в следующий раз.
///
/// Хранится по ключу «программа + основное имя цели».
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TargetSignature {
    /// Программа.
    pub app: AppId,
    /// Роль.
    pub role: Role,
    /// Имя.
    pub name: String,
    /// Постоянный идентификатор от разработчика программы.
    pub stable_id: Option<String>,
    /// Вид элемента на языке платформы.
    pub native_kind: Option<String>,
    /// Роли предков от окна до элемента.
    pub path: Vec<Role>,
}
