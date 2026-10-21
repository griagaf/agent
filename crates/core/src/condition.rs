//! Условия успеха действия и итог их проверки.

use serde::{Deserialize, Serialize};

use crate::observation::AppId;
use crate::plan::TargetSpec;

/// Признак того, что действие получилось.
///
/// Список закрыт: каждое условие ядро проверяет само, сравнивая наблюдения
/// до и после действия. Если условие не выражается этим списком, модель пишет
/// [`Condition::Unclear`], и проверку делает она же.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Condition {
    /// Появилось окно, которого не было до действия.
    WindowAppeared {
        /// Заголовок окна.
        title: TextMatch,
    },
    /// Окно закрылось.
    WindowClosed {
        /// Заголовок окна.
        title: TextMatch,
    },
    /// Впереди оказалось окно этой программы.
    ForegroundIs {
        /// Программа.
        app: AppId,
    },
    /// Фокус ввода в этом элементе.
    FocusIn {
        /// Элемент.
        target: TargetSpec,
    },
    /// Элемент появился.
    ElementAppeared {
        /// Элемент.
        target: TargetSpec,
    },
    /// Элемент пропал.
    ElementGone {
        /// Элемент.
        target: TargetSpec,
    },
    /// Элемент стал виден, например после прокрутки.
    Visible {
        /// Элемент.
        target: TargetSpec,
    },
    /// Элемент выбран.
    Selected {
        /// Элемент.
        target: TargetSpec,
    },
    /// Галочка или переключатель в нужном положении.
    Toggled {
        /// Элемент.
        target: TargetSpec,
        /// Нужное положение.
        on: bool,
    },
    /// В поле нужный текст.
    ValueMatches {
        /// Поле.
        field: TargetSpec,
        /// Ожидаемый текст.
        text: TextMatch,
    },
    /// Адрес в адресной строке браузера.
    AddressMatches {
        /// Ожидаемый адрес.
        text: TextMatch,
    },
    /// Человек выбрал ответ на вопрос.
    Answered,
    /// Выполнены все условия.
    All {
        /// Условия.
        of: Vec<Condition>,
    },
    /// Выполнено хотя бы одно условие.
    Any {
        /// Условия.
        of: Vec<Condition>,
    },
    /// Код проверить не может, проверку делает модель.
    Unclear {
        /// Что проверить, словами.
        hint: String,
    },
}

/// Сравнение текста без учёта регистра, разницы между «ё» и «е» и лишних пробелов.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TextMatch {
    /// Совпадает целиком.
    Exact(String),
    /// Содержит подстроку.
    Contains(String),
    /// Начинается с подстроки.
    StartsWith(String),
    /// Совпадает целиком с одним из вариантов.
    AnyOf(Vec<String>),
}

/// Итог проверки условия.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Verdict {
    /// Условие выполнено.
    Met,
    /// Условие ещё не выполнено.
    NotYet,
    /// Код не может судить: условие [`Condition::Unclear`] или элементов на экране нет.
    Unknown,
}
