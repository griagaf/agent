use std::env;

use anyhow::{Result, bail};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProviderKind {
    Anthropic,
    OpenAiCompat,
}

#[derive(Debug, Clone)]
pub struct Config {
    pub provider: ProviderKind,
    pub base_url: String,
    pub api_key: String,
    pub model: String,
}

/// Готовые связки «куда ходить и какой моделью»: имя пресета -> провайдер,
/// адрес, модель по умолчанию и переменная с ключом.
struct Preset {
    provider: ProviderKind,
    base_url: &'static str,
    model: &'static str,
    key_var: &'static str,
    key_required: bool,
}

const PRESETS: &[(&str, Preset)] = &[
    (
        "anthropic",
        Preset {
            provider: ProviderKind::Anthropic,
            base_url: "https://api.anthropic.com",
            model: "claude-opus-5",
            key_var: "ANTHROPIC_API_KEY",
            key_required: true,
        },
    ),
    (
        "ollama",
        Preset {
            provider: ProviderKind::OpenAiCompat,
            base_url: "http://localhost:11434/v1",
            model: "qwen3:8b",
            key_var: "OLLAMA_API_KEY",
            key_required: false,
        },
    ),
    (
        "lmstudio",
        Preset {
            provider: ProviderKind::OpenAiCompat,
            base_url: "http://localhost:1234/v1",
            model: "qwen2.5-7b-instruct",
            key_var: "LMSTUDIO_API_KEY",
            key_required: false,
        },
    ),
    (
        "groq",
        Preset {
            provider: ProviderKind::OpenAiCompat,
            base_url: "https://api.groq.com/openai/v1",
            model: "openai/gpt-oss-120b",
            key_var: "GROQ_API_KEY",
            key_required: true,
        },
    ),
    (
        "openrouter",
        Preset {
            provider: ProviderKind::OpenAiCompat,
            base_url: "https://openrouter.ai/api/v1",
            model: "qwen/qwen3-8b:free",
            key_var: "OPENROUTER_API_KEY",
            key_required: true,
        },
    ),
];

impl Config {
    /// Читает `.env` рядом с проектом, затем переменные окружения.
    /// Пресет задаётся через AGENT_PROVIDER, любое его поле перекрывается
    /// переменными AGENT_MODEL, AGENT_BASE_URL и AGENT_API_KEY.
    pub fn from_env() -> Result<Self> {
        let _ = dotenvy::dotenv();

        let name = env::var("AGENT_PROVIDER").unwrap_or_else(|_| "anthropic".to_string());
        let name = name.trim().to_lowercase();

        let Some((_, preset)) = PRESETS.iter().find(|(key, _)| *key == name) else {
            let known: Vec<&str> = PRESETS.iter().map(|(key, _)| *key).collect();
            bail!(
                "неизвестный AGENT_PROVIDER='{name}'. Доступные: {}",
                known.join(", ")
            );
        };

        let api_key = env::var("AGENT_API_KEY")
            .or_else(|_| env::var(preset.key_var))
            .unwrap_or_default();

        if preset.key_required && api_key.is_empty() {
            bail!(
                "не задан ключ доступа: пропишите {} в файле .env или в переменных окружения",
                preset.key_var
            );
        }

        Ok(Self {
            provider: preset.provider,
            base_url: env::var("AGENT_BASE_URL").unwrap_or_else(|_| preset.base_url.to_string()),
            api_key,
            model: env::var("AGENT_MODEL").unwrap_or_else(|_| preset.model.to_string()),
        })
    }
}
