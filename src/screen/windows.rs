use std::cmp::Reverse;

use anyhow::{Context, Result, bail};
use uiautomation::controls::ControlType;
use uiautomation::patterns::{UIInvokePattern, UIValuePattern};
use uiautomation::{UIAutomation, UIElement, UIMatcher};
use windows::Win32::UI::WindowsAndMessaging::{
    GetSystemMetrics, SM_CXVIRTUALSCREEN, SM_CYVIRTUALSCREEN, SM_XVIRTUALSCREEN, SM_YVIRTUALSCREEN,
};

use super::control_types;
use super::ranking;
use super::types::{Match, Rect, Target};

const SEARCH_WAIT: u64 = 2500;
const POLL_INTERVAL: u64 = 150;
// Семи уровней по умолчанию не хватает: меню лежат глубже, а DOM веб-окон — ещё глубже.
const SEARCH_DEPTH: u32 = 20;

pub fn find(target: &Target, limit: usize) -> Result<Vec<Match>> {
    let automation = connect()?;

    Ok(search(&automation, target, SEARCH_WAIT)?
        .into_iter()
        .map(|(_, description)| description)
        .take(limit)
        .collect())
}

/// Проверка для опроса: ждать нельзя, иначе каждый тик стоит секунды.
pub fn exists(target: &Target) -> Result<bool> {
    let automation = connect()?;
    if let Some(kind) = &target.control_type {
        control_types::parse(kind)?;
    }

    // Закрытое окно — это и есть «элемента нет», а не сбой поиска.
    Ok(search(&automation, target, 0).is_ok_and(|found| !found.is_empty()))
}

pub fn click(target: &Target) -> Result<()> {
    let automation = connect()?;

    best_match(&automation, target)?
        .get_pattern::<UIInvokePattern>()
        .with_context(|| format!("элемент {} нельзя нажать", target.name))?
        .invoke()
        .with_context(|| format!("не удалось нажать {}", target.name))
}

pub fn type_text(target: &Target, text: &str) -> Result<()> {
    let automation = connect()?;

    best_match(&automation, target)?
        .get_pattern::<UIValuePattern>()
        .with_context(|| format!("в элемент {} нельзя вписать текст", target.name))?
        .set_value(text)
        .with_context(|| format!("не удалось вписать текст в {}", target.name))
}

pub fn text_of(target: &Target) -> Result<String> {
    let automation = connect()?;
    let element = best_match(&automation, target)?;

    match element.get_pattern::<UIValuePattern>() {
        Ok(pattern) => pattern
            .get_value()
            .with_context(|| format!("не удалось прочитать текст из {}", target.name)),
        Err(_) => Ok(ranking::shorten(&element.get_name().unwrap_or_default())),
    }
}

fn connect() -> Result<UIAutomation> {
    UIAutomation::new().context("не удалось подключиться к UI Automation")
}

// Ранжирование общее для поиска и для действия: нажимать надо ровно туда, куда показали.
fn search(
    automation: &UIAutomation,
    target: &Target,
    wait: u64,
) -> Result<Vec<(UIElement, Match)>> {
    // Когда ничего не совпало, крейт отдаёт таймаут, а не пустой список.
    let found = build_matcher(automation, target, wait)?
        .find_all()
        .unwrap_or_default();

    let screen = virtual_screen();
    let mut ranked: Vec<(UIElement, Match)> = found
        .into_iter()
        .filter_map(|element| {
            describe(&element, &screen, &target.name).map(|description| (element, description))
        })
        .collect();

    ranked.sort_by_key(|(_, item)| (Reverse(item.score), item.name.chars().count()));

    Ok(ranked)
}

fn best_match(automation: &UIAutomation, target: &Target) -> Result<UIElement> {
    let Some((element, _)) = search(automation, target, SEARCH_WAIT)?.into_iter().next() else {
        bail!("на экране нет элемента {}", target.name);
    };

    Ok(element)
}

fn build_matcher(automation: &UIAutomation, target: &Target, wait: u64) -> Result<UIMatcher> {
    let mut matcher = automation
        .create_matcher()
        .contains_name(&target.name)
        .depth(SEARCH_DEPTH)
        .timeout(wait)
        .interval(POLL_INTERVAL);

    if let Some(kind) = &target.control_type {
        matcher = matcher.control_type(control_types::parse(kind)?);
    }
    if let Some(title) = &target.window {
        matcher = matcher.from(find_window(automation, title)?);
    }

    Ok(matcher)
}

fn find_window(automation: &UIAutomation, title: &str) -> Result<UIElement> {
    automation
        .create_matcher()
        .control_type(ControlType::Window)
        .contains_name(title)
        .timeout(SEARCH_WAIT)
        .interval(POLL_INTERVAL)
        .find_first()
        .with_context(|| format!("не нашлось открытого окна с заголовком {title}"))
}

fn describe(element: &UIElement, screen: &Rect, query: &str) -> Option<Match> {
    if element.is_offscreen().unwrap_or(true) {
        return None;
    }

    let rect = bounds(element)?;
    if !ranking::is_visible(&rect, screen) {
        return None;
    }

    let name = ranking::shorten(&element.get_name().unwrap_or_default());

    Some(Match {
        score: ranking::score(&name, query),
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
