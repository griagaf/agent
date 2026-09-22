use anyhow::{Result, bail};
use uiautomation::controls::ControlType;

// Словарь для модели: английские имена, чтобы не зависеть от языка Windows.
const CONTROL_TYPES: &[(&str, ControlType)] = &[
    ("button", ControlType::Button),
    ("menu", ControlType::Menu),
    ("menu_item", ControlType::MenuItem),
    ("edit", ControlType::Edit),
    ("text", ControlType::Text),
    ("list", ControlType::List),
    ("list_item", ControlType::ListItem),
    ("tab_item", ControlType::TabItem),
    ("check_box", ControlType::CheckBox),
    ("radio_button", ControlType::RadioButton),
    ("combo_box", ControlType::ComboBox),
    ("hyperlink", ControlType::Hyperlink),
    ("tree_item", ControlType::TreeItem),
    ("window", ControlType::Window),
];

pub(super) fn parse(name: &str) -> Result<ControlType> {
    let wanted = name.trim().to_lowercase().replace(['_', '-', ' '], "");

    let found = CONTROL_TYPES
        .iter()
        .find(|(key, _)| key.replace('_', "") == wanted)
        .map(|(_, control_type)| *control_type);

    let Some(control_type) = found else {
        let known: Vec<&str> = CONTROL_TYPES.iter().map(|(key, _)| *key).collect();
        bail!(
            "неизвестный вид элемента {name}. Допустимые: {}",
            known.join(", ")
        );
    };

    Ok(control_type)
}

pub(super) fn name(control_type: ControlType) -> String {
    CONTROL_TYPES
        .iter()
        .find(|(_, known)| *known == control_type)
        .map_or_else(
            || format!("{control_type:?}"),
            |(key, _)| (*key).to_string(),
        )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_spelling_variants() {
        let expected = ControlType::MenuItem;
        assert_eq!(parse("menu_item").unwrap(), expected);
        assert_eq!(parse(" Menu Item ").unwrap(), expected);
        assert_eq!(parse("MenuItem").unwrap(), expected);
    }

    #[test]
    fn explains_unknown_control_type() {
        let error = parse("кнопка").unwrap_err().to_string();
        assert!(
            error.contains("button"),
            "в ошибке нужен список допустимых: {error}"
        );
    }

    #[test]
    fn names_round_trip() {
        for (key, control_type) in CONTROL_TYPES {
            assert_eq!(name(*control_type), *key);
        }
    }
}
