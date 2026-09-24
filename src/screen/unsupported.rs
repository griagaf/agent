use anyhow::{Result, bail};

use super::types::{Match, Target};

const REASON: &str = "работа с элементами экрана пока сделана только для Windows";

pub fn find(_target: &Target, _limit: usize) -> Result<Vec<Match>> {
    bail!(REASON);
}

pub fn exists(_target: &Target) -> Result<bool> {
    bail!(REASON);
}

pub fn windows() -> Result<Vec<String>> {
    bail!(REASON);
}

pub fn click(_target: &Target) -> Result<()> {
    bail!(REASON);
}

pub fn type_text(_target: &Target, _text: &str) -> Result<()> {
    bail!(REASON);
}

pub fn text_of(_target: &Target) -> Result<String> {
    bail!(REASON);
}
