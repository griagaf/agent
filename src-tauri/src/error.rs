use serde::{Serialize, Serializer};

/// Всё, что уходит фронтенду: код — для логики окна, текст — для человека.
pub enum Error {
    EmptyQuestion,
    AskFailed,
    PointerFailed,
    StepFailed,
    ElementMissing,
    NotConfigured,
}

impl Error {
    fn code(&self) -> u16 {
        match self {
            Self::EmptyQuestion => 100,
            Self::AskFailed => 101,
            Self::PointerFailed => 102,
            Self::StepFailed => 103,
            Self::ElementMissing => 104,
            Self::NotConfigured => 105,
        }
    }

    fn message(&self) -> &'static str {
        match self {
            Self::EmptyQuestion => "Напишите, что нужно сделать.",
            Self::AskFailed => "Не получилось выполнить запрос. Попробуйте ещё раз.",
            Self::PointerFailed => "Не получилось показать указатель.",
            Self::StepFailed => "Шаг не получилось выполнить.",
            Self::ElementMissing => "Не вижу этот элемент на экране. Откройте нужное окно.",
            Self::NotConfigured => "Не настроен доступ к модели: проверьте файл .env.",
        }
    }
}

#[derive(Serialize)]
struct Payload {
    code: u16,
    message: &'static str,
}

impl Serialize for Error {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        Payload {
            code: self.code(),
            message: self.message(),
        }
        .serialize(serializer)
    }
}
