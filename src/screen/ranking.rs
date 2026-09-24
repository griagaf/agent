use super::types::Rect;

// Подписью бывает весь текст сообщения или письма — модели столько не нужно.
const MAX_NAME: usize = 120;

// Запрос часто встречается внутри длинного чужого текста, и такое совпадение бесполезно.
pub(super) fn score(name: &str, query: &str) -> u8 {
    let name = name.trim().to_lowercase();
    let query = query.trim().to_lowercase();

    if name == query {
        3
    } else if name.starts_with(&query) {
        2
    } else {
        1
    }
}

pub(super) fn shorten(name: &str) -> String {
    let collapsed = name.split_whitespace().collect::<Vec<_>>().join(" ");
    if collapsed.chars().count() <= MAX_NAME {
        return collapsed;
    }

    collapsed.chars().take(MAX_NAME).collect::<String>() + "…"
}

// Прокрученный за край списка элемент UI Automation всё ещё считает видимым.
pub(super) fn is_visible(rect: &Rect, screen: &Rect) -> bool {
    rect.right > rect.left
        && rect.bottom > rect.top
        && rect.right > screen.left
        && rect.left < screen.right
        && rect.bottom > screen.top
        && rect.top < screen.bottom
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ranks_exact_label_above_accidental_substring() {
        assert_eq!(score("Пуск", "пуск"), 3);
        assert_eq!(score("Пускай", "пуск"), 2);
        assert_eq!(score("наши выпускники", "пуск"), 1);
    }

    #[test]
    fn hides_elements_scrolled_out_of_the_screen() {
        let screen = Rect {
            left: 0,
            top: 0,
            right: 1920,
            bottom: 1080,
        };
        let visible = Rect {
            left: 684,
            top: 1020,
            right: 740,
            bottom: 1080,
        };
        let scrolled_away = Rect {
            left: 686,
            top: -12941,
            right: 1920,
            bottom: -12527,
        };

        assert!(is_visible(&visible, &screen));
        assert!(!is_visible(&scrolled_away, &screen));
    }

    #[test]
    fn collapses_and_cuts_long_labels() {
        assert_eq!(shorten("  Файл\n  меню "), "Файл меню");
        assert_eq!(shorten(&"я".repeat(200)).chars().count(), MAX_NAME + 1);
    }
}
