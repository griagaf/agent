//! План задачи: черновик, который составляет модель, и принятый план с номерами пунктов.

use std::borrow::Cow;

use serde::{Deserialize, Serialize};

use crate::condition::{Condition, TextMatch};
use crate::observation::{AppId, Platform, Role};

/// План в том виде, в каком его возвращает модель.
///
/// Номера пунктов в черновике нет: их раздаёт ядро при приёмке, см. [`Plan::accept`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PlanDraft {
    /// Название задачи для панели плана.
    pub task_title: String,
    /// Пункты по порядку.
    pub steps: Vec<StepDraft>,
}

/// Пункт плана: одна строка в панели плана.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StepDraft {
    /// Название пункта. Во всех местах интерфейса пишется одинаково.
    pub title: String,
    /// Действия человека. Каждое показывается своей карточкой.
    pub actions: Vec<Action>,
}

/// Одно действие человека. Ему соответствует одна карточка подсказки.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Action {
    /// Что обвести на экране. Нет у вопросов и у нажатий клавиш.
    #[serde(default)]
    pub target: Option<TargetSpec>,
    /// Что нужно сделать.
    pub kind: ActionKind,
    /// По какому признаку ядро поймёт, что действие получилось.
    pub success: Condition,
    /// Текст карточки.
    pub card: CardText,
    /// Видела ли модель цель на экране, когда составляла план.
    pub certainty: Certainty,
}

/// Описание цели словами элементов интерфейса. Координат в нём нет.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TargetSpec {
    /// Допустимые роли. Пустой список означает любую роль.
    #[serde(default)]
    pub roles: Vec<Role>,
    /// Варианты имени: на разных языках и синонимы. Первый вариант основной.
    pub names: Vec<String>,
    /// В какой программе и в каком окне искать.
    #[serde(default)]
    pub scope: Scope,
    /// Примерная область окна.
    #[serde(default)]
    pub region: Option<Region>,
    /// Имя соседнего элемента: «кнопка рядом с надписью „Кому“».
    #[serde(default)]
    pub near: Option<String>,
}

/// Где искать цель.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct Scope {
    /// Программа.
    #[serde(default)]
    pub app: Option<AppId>,
    /// Заголовок окна.
    #[serde(default)]
    pub window: Option<TextMatch>,
}

/// Примерная область окна.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Region {
    /// Верх.
    Top,
    /// Низ.
    Bottom,
    /// Левый край.
    Left,
    /// Правый край.
    Right,
    /// Середина.
    Center,
    /// Левый верхний угол.
    TopLeft,
    /// Правый верхний угол.
    TopRight,
    /// Левый нижний угол.
    BottomLeft,
    /// Правый нижний угол.
    BottomRight,
}

/// Что должен сделать человек.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ActionKind {
    /// Щёлкнуть левой кнопкой мыши.
    Click,
    /// Дважды щёлкнуть левой кнопкой мыши.
    DoubleClick,
    /// Щёлкнуть правой кнопкой мыши.
    RightClick,
    /// Напечатать текст.
    TypeText {
        /// Какой текст.
        text: TextToType,
    },
    /// Нажать клавишу или сочетание: зажать `hold`, затем нажать `key`.
    PressKey {
        /// Основная клавиша.
        key: Key,
        /// Клавиши, которые нужно держать.
        #[serde(default)]
        hold: Vec<Modifier>,
    },
    /// Прокрутить колёсиком мыши.
    Scroll {
        /// Направление.
        direction: ScrollDir,
    },
    /// Выбрать ответ на вопрос. Вариант «Не знаю» ядро добавляет само.
    Ask {
        /// Вопрос.
        question: String,
        /// Варианты ответа.
        options: Vec<String>,
    },
}

/// Текст, который печатает человек.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TextToType {
    /// Известен заранее: адрес сайта, слово из задачи.
    Exact(String),
    /// Человек печатает своё.
    Own {
        /// Секретный ввод, например пароль. Пока он идёт, нажатия клавиш
        /// не отслеживаются и экран не снимается.
        secret: bool,
    },
}

/// Клавиша по смыслу, а не по надписи.
///
/// Надпись на клавише и сама клавиша зависят от платформы: см. [`Key::label`]
/// и [`Key::exists_on`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Key {
    /// Enter.
    Enter,
    /// Backspace.
    Backspace,
    /// Пробел.
    Space,
    /// Esc.
    Escape,
    /// Tab.
    Tab,
    /// Delete.
    Delete,
    /// Insert.
    Insert,
    /// Стрелка вверх.
    Up,
    /// Стрелка вниз.
    Down,
    /// Стрелка влево.
    Left,
    /// Стрелка вправо.
    Right,
    /// Home.
    Home,
    /// End.
    End,
    /// Page Up.
    PageUp,
    /// Page Down.
    PageDown,
    /// Print Screen.
    PrintScreen,
    /// Функциональная клавиша F1–F12.
    F(u8),
    /// Клавиша с буквой, цифрой или знаком; символ берётся из латинской раскладки.
    /// Сочетания привязаны к месту клавиши: Ctrl+C в русской раскладке
    /// нажимается той же клавишей, что и в английской.
    Char(char),
}

impl Key {
    /// Надпись на клавише для этой платформы.
    pub fn label(self, platform: Platform) -> Cow<'static, str> {
        let label = match (self, platform) {
            (Self::Enter, Platform::Windows) => "Enter",
            (Self::Enter, Platform::MacOs) => "Return",
            (Self::Backspace, Platform::Windows) => "Backspace",
            (Self::Backspace, Platform::MacOs) => "Delete",
            (Self::Delete, Platform::Windows) => "Delete",
            (Self::Delete, Platform::MacOs) => "Forward Delete",
            (Self::Space, _) => "Пробел",
            (Self::Escape, _) => "Esc",
            (Self::Tab, _) => "Tab",
            (Self::Insert, _) => "Insert",
            (Self::Up, _) => "↑",
            (Self::Down, _) => "↓",
            (Self::Left, _) => "←",
            (Self::Right, _) => "→",
            (Self::Home, _) => "Home",
            (Self::End, _) => "End",
            (Self::PageUp, _) => "Page Up",
            (Self::PageDown, _) => "Page Down",
            (Self::PrintScreen, _) => "Print Screen",
            (Self::F(number), _) => return Cow::Owned(format!("F{number}")),
            (Self::Char(symbol), _) => return Cow::Owned(symbol.to_uppercase().collect()),
        };
        Cow::Borrowed(label)
    }

    /// Есть ли такая клавиша на клавиатуре этой платформы.
    ///
    /// На Mac нет Insert и Print Screen. Home, End, Page Up и Page Down
    /// на ноутбуках Mac нажимаются вместе с Fn, но считаются существующими.
    /// Функциональные клавиши допустимы от F1 до F12, символьные — только
    /// видимые символы латинской раскладки.
    pub fn exists_on(self, platform: Platform) -> bool {
        match (self, platform) {
            (Self::Insert | Self::PrintScreen, Platform::MacOs) => false,
            (Self::F(number), _) => (1..=12).contains(&number),
            (Self::Char(symbol), _) => symbol.is_ascii_graphic(),
            _ => true,
        }
    }
}

/// Клавиша, которую держат при сочетании. Названа по смыслу, а не по надписи.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Modifier {
    /// Главная клавиша команд: Ctrl в Windows, Command в macOS.
    Primary,
    /// Shift.
    Shift,
    /// Alt в Windows, Option в macOS.
    Alt,
    /// Win в Windows, Control в macOS.
    System,
}

impl Modifier {
    /// Надпись на клавише для этой платформы.
    pub fn label(self, platform: Platform) -> &'static str {
        match (self, platform) {
            (Self::Primary, Platform::Windows) => "Ctrl",
            (Self::Primary, Platform::MacOs) => "Command",
            (Self::Shift, _) => "Shift",
            (Self::Alt, Platform::Windows) => "Alt",
            (Self::Alt, Platform::MacOs) => "Option",
            (Self::System, Platform::Windows) => "Win",
            (Self::System, Platform::MacOs) => "Control",
        }
    }
}

/// Направление прокрутки.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ScrollDir {
    /// Вниз.
    Down,
    /// Вверх.
    Up,
}

/// Текст карточки подсказки.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CardText {
    /// Заголовок: одно действие, глагол первым.
    pub title: String,
    /// Пояснение. Метку `{где}` ядро заменяет на расположение найденной цели.
    pub body: String,
}

/// Насколько модель уверена в описании цели.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Certainty {
    /// Цель была на экране, когда составлялся план.
    Exact,
    /// Нужный экран ещё не открыт, имена взяты из знаний модели.
    Sketch,
}

/// Номер пункта плана. Сохраняется, когда план перестраивается.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct StepId(pub u32);

/// Принятый план с номерами пунктов.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Plan {
    /// Версия плана. Растёт при каждой перестройке.
    pub version: u32,
    /// Название задачи.
    pub task_title: String,
    /// Пункты по порядку.
    pub steps: Vec<PlanStep>,
}

/// Пункт принятого плана.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PlanStep {
    /// Номер пункта.
    pub id: StepId,
    /// Название.
    pub title: String,
    /// Действия человека.
    pub actions: Vec<Action>,
}

impl Plan {
    /// Принимает черновик модели: раздаёт пунктам номера подряд, начиная с `first_id`.
    pub fn accept(draft: PlanDraft, version: u32, first_id: u32) -> Self {
        let steps = (first_id..)
            .zip(draft.steps)
            .map(|(id, step)| PlanStep {
                id: StepId(id),
                title: step.title,
                actions: step.actions,
            })
            .collect();
        Self {
            version,
            task_title: draft.task_title,
            steps,
        }
    }
}
