pub mod find_element;
pub mod find_file;

use std::sync::Arc;

use anyhow::Result;
use serde_json::Value;

use crate::llm::ToolSpec;

/// Инструмент выполняется синхронно: работа тут файловая, а не сетевая.
/// Агент сам уводит вызов в spawn_blocking.
pub trait Tool: Send + Sync {
    fn spec(&self) -> ToolSpec;
    fn call(&self, input: &Value) -> Result<String>;
}

pub struct ToolBox {
    tools: Vec<Arc<dyn Tool>>,
}

impl ToolBox {
    pub fn with_defaults() -> Self {
        Self {
            tools: vec![
                Arc::new(find_file::FindFile),
                Arc::new(find_element::FindElement),
            ],
        }
    }

    pub fn specs(&self) -> Vec<ToolSpec> {
        self.tools.iter().map(|tool| tool.spec()).collect()
    }

    pub fn find(&self, name: &str) -> Option<Arc<dyn Tool>> {
        self.tools
            .iter()
            .find(|tool| tool.spec().name == name)
            .cloned()
    }
}
