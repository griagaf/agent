use std::fs;
use std::path::PathBuf;

use agent::tools::ToolBox;
use serde_json::{Value, json};

/// Готовит настоящее дерево папок во временном каталоге.
fn make_tree(label: &str) -> PathBuf {
    let root = std::env::temp_dir().join(format!("agent-test-{label}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);

    let nested = root.join("Документы").join("учёба");
    fs::create_dir_all(&nested).unwrap();
    fs::write(nested.join("Отчёт по практике.docx"), vec![0u8; 2048]).unwrap();
    fs::write(nested.join("смета.xlsx"), b"x").unwrap();

    root
}

fn find(input: Value) -> Value {
    let tools = ToolBox::with_defaults();
    let tool = tools.find("find_file").expect("инструмент зарегистрирован");
    serde_json::from_str(&tool.call(&input).unwrap()).unwrap()
}

#[test]
fn returns_path_and_size_of_matching_file() {
    let root = make_tree("hit");

    let result = find(json!({
        "query": "отчёт практика",
        "root": root.to_str().unwrap(),
    }));

    assert_eq!(result["found"], 1);
    let hit = &result["results"][0];
    assert_eq!(hit["name"], "Отчёт по практике.docx");
    assert_eq!(hit["size_bytes"], 2048);
    assert_eq!(hit["size"], "2.0 КБ");
    assert!(
        hit["path"].as_str().unwrap().ends_with("Отчёт по практике.docx"),
        "путь должен быть полным: {hit}"
    );

    fs::remove_dir_all(&root).unwrap();
}

#[test]
fn explains_when_nothing_matches() {
    let root = make_tree("miss");

    let result = find(json!({
        "query": "презентация",
        "root": root.to_str().unwrap(),
    }));

    assert_eq!(result["found"], 0);
    assert!(result["hint"].is_string());

    fs::remove_dir_all(&root).unwrap();
}
