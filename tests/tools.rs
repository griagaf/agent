use agent::tools::ToolBox;

/// Схему видит только модель, поэтому опечатка в ней всплыла бы на живом запросе.
#[test]
fn every_tool_is_described_for_the_model() {
    let specs = ToolBox::with_defaults().specs();
    assert!(specs.len() >= 3, "инструменты потерялись: {}", specs.len());

    for spec in &specs {
        assert!(!spec.name.trim().is_empty(), "инструмент без имени");
        assert!(
            !spec.description.trim().is_empty(),
            "{} без описания",
            spec.name
        );
        assert_eq!(
            spec.input_schema["type"], "object",
            "{} описан не как объект",
            spec.name
        );
        assert!(
            spec.input_schema["properties"].is_object(),
            "{} без списка полей",
            spec.name
        );
    }
}

#[test]
fn tool_names_do_not_repeat() {
    let specs = ToolBox::with_defaults().specs();
    let mut names: Vec<&str> = specs.iter().map(|spec| spec.name.as_str()).collect();
    let total = names.len();

    names.sort_unstable();
    names.dedup();

    assert_eq!(
        names.len(),
        total,
        "имена инструментов повторяются: {names:?}"
    );
}
