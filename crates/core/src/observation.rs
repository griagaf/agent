//! Наблюдение экрана: окна и элементы интерфейса в том виде, в каком их отдаёт платформа.

use serde::{Deserialize, Serialize};

/// Номер поколения снимка экрана.
///
/// Каждый ответ модели или кода несёт номер снимка, по которому он посчитан.
/// Ответ со старым номером относится к уже изменившемуся экрану и отбрасывается.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Epoch(pub u64);

/// Ссылка на элемент внутри одного наблюдения.
///
/// Между наблюдениями не переносится. Чтобы узнать тот же элемент позже,
/// используется [`TargetSignature`](crate::grounding::TargetSignature).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ElementRef(pub u32);

/// Прямоугольник в физических пикселях виртуального рабочего стола.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct PxRect {
    /// Левый край.
    pub x: i32,
    /// Верхний край.
    pub y: i32,
    /// Ширина.
    pub w: u32,
    /// Высота.
    pub h: u32,
}

/// Имя программы в том виде, в каком его даёт платформа:
/// `explorer.exe` в Windows, `com.apple.finder` в macOS.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct AppId(pub String);

/// Операционная система. Нужна там, где различие видит человек: в подписях клавиш.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Platform {
    /// Windows.
    Windows,
    /// macOS.
    MacOs,
}

/// Снимок того, что сейчас на экране.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Observation {
    /// Номер снимка.
    pub epoch: Epoch,
    /// Открытые окна.
    pub windows: Vec<WindowInfo>,
    /// Именованные или нажимаемые элементы активного окна и панели задач.
    pub elements: Vec<Element>,
    /// Насколько полно платформа описала элементы.
    pub quality: TreeQuality,
}

/// Окно на экране.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WindowInfo {
    /// Заголовок.
    pub title: String,
    /// Программа, которой принадлежит окно.
    pub app: AppId,
    /// Положение и размер.
    pub rect: PxRect,
    /// Номер монитора.
    pub monitor: u32,
    /// Окно впереди и принимает ввод.
    pub foreground: bool,
    /// Система не даёт читать элементы окна: программа запущена
    /// с правами администратора или не выдано разрешение.
    pub unreadable: bool,
}

/// Элемент интерфейса: кнопка, поле, строка списка.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Element {
    /// Ссылка на элемент в этом наблюдении.
    pub id: ElementRef,
    /// Индекс окна в [`Observation::windows`].
    pub window: u16,
    /// Родительский элемент.
    pub parent: Option<ElementRef>,
    /// Роль.
    pub role: Role,
    /// Видимое имя или подпись.
    pub name: String,
    /// Постоянный идентификатор от разработчика программы
    /// (`AutomationId` в Windows, `AXIdentifier` в macOS).
    pub stable_id: Option<String>,
    /// Вид элемента на языке платформы (`ClassName` в Windows, `AXSubrole` в macOS).
    pub native_kind: Option<String>,
    /// Текст поля. У поля пароля всегда `None`.
    pub value: Option<String>,
    /// Положение и размер.
    pub rect: PxRect,
    /// Элемент доступен для нажатия или ввода.
    pub enabled: bool,
    /// Элемент за пределами видимой области.
    pub offscreen: bool,
    /// В элементе фокус ввода.
    pub focused: bool,
    /// Элемент выбран: строка списка, вкладка.
    pub selected: bool,
    /// Поле пароля.
    pub password: bool,
    /// Положение галочки или переключателя.
    pub toggled: Option<bool>,
}

/// Роль элемента интерфейса.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Role {
    /// Кнопка.
    Button,
    /// Ссылка.
    Link,
    /// Поле ввода.
    Edit,
    /// Галочка.
    CheckBox,
    /// Переключатель.
    RadioButton,
    /// Выпадающий список.
    ComboBox,
    /// Строка списка.
    ListItem,
    /// Узел дерева.
    TreeItem,
    /// Пункт меню.
    MenuItem,
    /// Вкладка.
    Tab,
    /// Текст.
    Text,
    /// Картинка.
    Image,
    /// Документ или веб-страница.
    Document,
    /// Окно.
    Window,
    /// Любая другая роль.
    Other,
}

/// Насколько полно платформа описала элементы окна.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TreeQuality {
    /// У элементов есть имена и роли: цель можно найти кодом.
    Good,
    /// Много безымянных элементов: цель выбирает модель из списка кандидатов.
    Sparse,
    /// Элементов нет: цель ищется по снимку экрана.
    Empty,
}
