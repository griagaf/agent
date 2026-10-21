//! Черновик плана от модели: разбор, приёмка и обратная сериализация.

use mentor_core::observation::Platform;
use mentor_core::plan::{
    ActionKind, Key, Modifier, Plan, PlanDraft, Scope, StepId, TargetSpec, TextToType,
};

/// Ответ модели на просьбу «найди файл, который я скачал».
const FIND_DOWNLOAD: &str = include_str!("../testdata/plan_find_download.json");

fn draft() -> PlanDraft {
    serde_json::from_str(FIND_DOWNLOAD).expect("пример плана разбирается")
}

#[test]
fn accept_numbers_steps_from_first_id() {
    let plan = Plan::accept(draft(), 2, 7);

    assert_eq!(plan.version, 2);
    assert_eq!(plan.task_title, "Найти скачанный файл");
    let ids: Vec<_> = plan.steps.iter().map(|step| step.id).collect();
    assert_eq!(ids, [StepId(7), StepId(8), StepId(9)]);
}

#[test]
fn one_step_can_hold_several_cards() {
    let plan = Plan::accept(draft(), 1, 1);
    let open_mail = &plan.steps[2];

    assert_eq!(open_mail.actions.len(), 3);
    assert!(open_mail.actions[2].target.is_none());
}

#[test]
fn draft_survives_json_roundtrip() {
    let draft = draft();
    let json = serde_json::to_string(&draft).unwrap();

    assert_eq!(serde_json::from_str::<PlanDraft>(&json).unwrap(), draft);
}

#[test]
fn target_fields_have_defaults() {
    let target: TargetSpec = serde_json::from_str(r#"{ "names": ["Загрузки"] }"#).unwrap();

    assert!(target.roles.is_empty());
    assert_eq!(target.scope, Scope::default());
    assert_eq!(target.region, None);
    assert_eq!(target.near, None);
}

#[test]
fn keys_and_chords_parse() {
    let parse = |json| serde_json::from_str::<ActionKind>(json).unwrap();

    assert_eq!(
        parse(r#"{ "type": "press_key", "key": "enter" }"#),
        ActionKind::PressKey {
            key: Key::Enter,
            hold: vec![]
        }
    );
    assert_eq!(
        parse(r#"{ "type": "press_key", "key": { "char": "c" }, "hold": ["primary"] }"#),
        ActionKind::PressKey {
            key: Key::Char('c'),
            hold: vec![Modifier::Primary]
        }
    );
    assert_eq!(
        parse(r#"{ "type": "press_key", "key": { "f": 5 } }"#),
        ActionKind::PressKey {
            key: Key::F(5),
            hold: vec![]
        }
    );
    assert_eq!(
        parse(r#"{ "type": "right_click" }"#),
        ActionKind::RightClick
    );
}

#[test]
fn own_text_can_be_secret() {
    let kind: ActionKind =
        serde_json::from_str(r#"{ "type": "type_text", "text": { "own": { "secret": true } } }"#)
            .unwrap();

    assert_eq!(
        kind,
        ActionKind::TypeText {
            text: TextToType::Own { secret: true }
        }
    );
}

#[test]
fn modifier_labels_follow_platform() {
    use Modifier::*;
    use Platform::*;

    let labels: Vec<_> = [Primary, Shift, Alt, System]
        .into_iter()
        .map(|modifier| (modifier.label(Windows), modifier.label(MacOs)))
        .collect();

    assert_eq!(
        labels,
        [
            ("Ctrl", "Command"),
            ("Shift", "Shift"),
            ("Alt", "Option"),
            ("Win", "Control"),
        ]
    );
}

#[test]
fn key_labels_follow_platform() {
    use Key::*;
    use Platform::*;

    let keys = [
        Enter,
        Backspace,
        Delete,
        Space,
        Escape,
        Tab,
        Insert,
        Up,
        Down,
        Left,
        Right,
        Home,
        End,
        PageUp,
        PageDown,
        PrintScreen,
        F(5),
        Char('c'),
        Char('/'),
    ];
    let labels: Vec<_> = keys
        .into_iter()
        .map(|key| {
            (
                key.label(Windows).into_owned(),
                key.label(MacOs).into_owned(),
            )
        })
        .collect();

    let expected = [
        ("Enter", "Return"),
        ("Backspace", "Delete"),
        ("Delete", "Forward Delete"),
        ("Пробел", "Пробел"),
        ("Esc", "Esc"),
        ("Tab", "Tab"),
        ("Insert", "Insert"),
        ("↑", "↑"),
        ("↓", "↓"),
        ("←", "←"),
        ("→", "→"),
        ("Home", "Home"),
        ("End", "End"),
        ("Page Up", "Page Up"),
        ("Page Down", "Page Down"),
        ("Print Screen", "Print Screen"),
        ("F5", "F5"),
        ("C", "C"),
        ("/", "/"),
    ];
    assert_eq!(
        labels,
        expected.map(|(windows, mac)| (windows.to_owned(), mac.to_owned()))
    );
}

#[test]
fn mac_has_no_insert_and_print_screen() {
    for key in [Key::Insert, Key::PrintScreen] {
        assert!(key.exists_on(Platform::Windows));
        assert!(!key.exists_on(Platform::MacOs));
    }
    for key in [Key::Enter, Key::Home, Key::PageDown] {
        assert!(key.exists_on(Platform::MacOs));
    }
}

#[test]
fn function_keys_go_from_f1_to_f12() {
    let valid: Vec<_> = [0, 1, 12, 13]
        .into_iter()
        .map(|number| Key::F(number).exists_on(Platform::Windows))
        .collect();

    assert_eq!(valid, [false, true, true, false]);
}

#[test]
fn char_keys_are_visible_latin_symbols() {
    for symbol in ['c', 'Z', '7', '/'] {
        assert!(Key::Char(symbol).exists_on(Platform::Windows), "{symbol}");
    }
    for symbol in ['ё', ' ', '\n'] {
        assert!(
            !Key::Char(symbol).exists_on(Platform::Windows),
            "{symbol:?}"
        );
    }
}
