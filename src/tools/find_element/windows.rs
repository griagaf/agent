use std::cmp::Reverse;

use anyhow::{Context, Result};
use uiautomation::controls::ControlType;
use uiautomation::{UIAutomation, UIElement, UIMatcher};
use windows::Win32::UI::WindowsAndMessaging::{
    GetSystemMetrics, SM_CXVIRTUALSCREEN, SM_CYVIRTUALSCREEN, SM_XVIRTUALSCREEN, SM_YVIRTUALSCREEN,
};

use super::common::{Match, Query, Rect};
use super::control_types;

const SEARCH_TIMEOUT: u64 = 2500;
const POLL_INTERVAL: u64 = 150;
// Семи уровней по умолчанию не хватает: меню и диалоги лежат глубже.
const SEARCH_DEPTH: u32 = 12;
// Подписью бывает весь текст сообщения или письма — модели столько не нужно.
const MAX_NAME: usize = 120;

pub(super) fn find(query: &Query) -> Result<Vec<Match>> {
    let automation = UIAutomation::new().context("не удалось подключиться к UI Automation")?;

    // Когда ничего не совпало, крейт отдаёт таймаут, а не пустой список.
    let found = build_matcher(&automation, query)?
        .find_all()
        .unwrap_or_default();

    let screen = virtual_screen();
    let mut matches: Vec<Match> = found
        .iter()
        .filter_map(|element| describe(element, &screen, &query.name))
        .collect();

    matches.sort_by_key(|item| (Reverse(item.score), item.name.chars().count()));
    matches.truncate(query.limit);

    Ok(matches)
}

fn build_matcher(automation: &UIAutomation, query: &Query) -> Result<UIMatcher> {
    let mut matcher = automation
        .create_matcher()
        .contains_name(&query.name)
        .depth(SEARCH_DEPTH)
        .timeout(SEARCH_TIMEOUT)
        .interval(POLL_INTERVAL);

    if let Some(kind) = &query.control_type {
        matcher = matcher.control_type(control_types::parse(kind)?);
    }
    if let Some(title) = &query.window {
        matcher = matcher.from(find_window(automation, title)?);
    }

    Ok(matcher)
}

fn find_window(automation: &UIAutomation, title: &str) -> Result<UIElement> {
    automation
        .create_matcher()
        .control_type(ControlType::Window)
        .contains_name(title)
        .timeout(SEARCH_TIMEOUT)
        .interval(POLL_INTERVAL)
        .find_first()
        .with_context(|| format!("не нашлось открытого окна с заголовком {title}"))
}

fn describe(element: &UIElement, screen: &Rect, query: &str) -> Option<Match> {
    if element.is_offscreen().unwrap_or(true) {
        return None;
    }

    let rect = bounds(element)?;
    if !is_visible(&rect, screen) {
        return None;
    }

    let name = shorten(&element.get_name().unwrap_or_default());

    Some(Match {
        score: score(&name, query),
        name,
        control_type: element
            .get_control_type()
            .map(control_types::name)
            .unwrap_or_default(),
        enabled: element.is_enabled().unwrap_or(false),
        rect,
    })
}

fn bounds(element: &UIElement) -> Option<Rect> {
    let rect = element.get_bounding_rectangle().ok()?;

    Some(Rect {
        left: rect.get_left(),
        top: rect.get_top(),
        right: rect.get_right(),
        bottom: rect.get_bottom(),
    })
}

// Прокрученный за край списка элемент UI Automation всё ещё считает видимым.
fn is_visible(rect: &Rect, screen: &Rect) -> bool {
    rect.right > rect.left
        && rect.bottom > rect.top
        && rect.right > screen.left
        && rect.left < screen.right
        && rect.bottom > screen.top
        && rect.top < screen.bottom
}

fn virtual_screen() -> Rect {
    // Безопасно: чтение системных метрик, без указателей и хэндлов.
    let (left, top, width, height) = unsafe {
        (
            GetSystemMetrics(SM_XVIRTUALSCREEN),
            GetSystemMetrics(SM_YVIRTUALSCREEN),
            GetSystemMetrics(SM_CXVIRTUALSCREEN),
            GetSystemMetrics(SM_CYVIRTUALSCREEN),
        )
    };

    Rect {
        left,
        top,
        right: left + width,
        bottom: top + height,
    }
}

fn shorten(name: &str) -> String {
    let collapsed = name.split_whitespace().collect::<Vec<_>>().join(" ");
    if collapsed.chars().count() <= MAX_NAME {
        return collapsed;
    }

    collapsed.chars().take(MAX_NAME).collect::<String>() + "…"
}

// Запрос часто встречается внутри длинного чужого текста, и такое совпадение бесполезно.
fn score(name: &str, query: &str) -> u8 {
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
