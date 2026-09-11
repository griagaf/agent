use std::collections::HashSet;
use std::path::PathBuf;
use std::time::{Duration, Instant, SystemTime};

use anyhow::{Result, bail};
use serde::Serialize;
use serde_json::{Value, json};
use walkdir::{DirEntry, WalkDir};

use super::Tool;
use crate::llm::ToolSpec;

const MAX_DEPTH: usize = 12;
const TIME_LIMIT: Duration = Duration::from_secs(25);
/// Сколько совпадений собираем до сортировки: ранжировать можно только то,
/// что уже нашли, поэтому берём с запасом относительно max_results.
const CANDIDATE_LIMIT: usize = 400;

/// Каталоги, где лежит система, кеши и мусор: пользовательских файлов там нет,
/// а обход стоит десятков секунд.
const SKIP_DIRS: &[&str] = &[
    "appdata",
    "windows",
    "program files",
    "program files (x86)",
    "programdata",
    "$recycle.bin",
    "system volume information",
    "node_modules",
    "target",
    "venv",
    "__pycache__",
    "onedrivetemp",
];

pub struct FindFile;

#[derive(Serialize)]
struct Hit {
    name: String,
    path: String,
    size: String,
    size_bytes: u64,
    modified: String,
    #[serde(skip)]
    score: u8,
    #[serde(skip)]
    modified_at: SystemTime,
}

impl Tool for FindFile {
    fn spec(&self) -> ToolSpec {
        ToolSpec {
            name: "find_file".to_string(),
            description: "Ищет файл на компьютере пользователя по части имени и возвращает \
                полный путь, размер и дату изменения. По умолчанию просматривает папки \
                пользователя (Рабочий стол, Документы, Загрузки и остальную домашнюю папку), \
                системные каталоги пропускает."
                .to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "query": {
                        "type": "string",
                        "description": "Часть имени файла. Несколько слов через пробел ищутся \
                            все сразу: 'отчёт практика' найдёт 'Отчёт по практике.docx'."
                    },
                    "root": {
                        "type": "string",
                        "description": "Необязательно: конкретная папка, в которой искать."
                    },
                    "max_results": {
                        "type": "integer",
                        "description": "Сколько файлов вернуть, по умолчанию 10."
                    }
                },
                "required": ["query"]
            }),
        }
    }

    fn call(&self, input: &Value) -> Result<String> {
        let query = input["query"].as_str().unwrap_or_default().trim();
        if query.is_empty() {
            bail!("не указано, что искать");
        }

        let (tokens, extension) = split_query(query);
        let limit = input["max_results"].as_u64().unwrap_or(10).clamp(1, 50) as usize;

        let roots = match input["root"].as_str() {
            Some(path) => {
                let root = PathBuf::from(path);
                if !root.is_dir() {
                    bail!("папки '{path}' не существует");
                }
                vec![root]
            }
            None => user_roots(),
        };

        let (hits, stopped_early) = search(&roots, &tokens, extension.as_deref(), limit);

        if hits.is_empty() {
            return Ok(json!({
                "found": 0,
                "results": [],
                "searched_in": roots.iter().map(display_path).collect::<Vec<_>>(),
                "hint": "Ничего не найдено. Возможно, файл называется иначе или лежит \
                    вне папок пользователя — тогда стоит переспросить и передать root."
            })
            .to_string());
        }

        Ok(json!({
            "found": hits.len(),
            "results": hits,
            "search_incomplete": stopped_early,
        })
        .to_string())
    }
}

/// Порядок важен: сначала места, где файл лежит с наибольшей вероятностью,
/// затем остальная домашняя папка. Повторно уже пройденное не обходим.
fn user_roots() -> Vec<PathBuf> {
    [
        dirs::desktop_dir(),
        dirs::document_dir(),
        dirs::download_dir(),
        dirs::picture_dir(),
        dirs::video_dir(),
        dirs::audio_dir(),
        dirs::home_dir(),
    ]
    .into_iter()
    .flatten()
    .filter(|path| path.is_dir())
    .collect()
}

fn search(
    roots: &[PathBuf],
    tokens: &[String],
    extension: Option<&str>,
    limit: usize,
) -> (Vec<Hit>, bool) {
    let deadline = Instant::now() + TIME_LIMIT;
    let mut visited: HashSet<PathBuf> = HashSet::new();
    let mut hits: Vec<Hit> = Vec::new();
    let mut stopped_early = false;

    for root in roots {
        let walker = WalkDir::new(root)
            .max_depth(MAX_DEPTH)
            .into_iter()
            .filter_entry(|entry| keep_dir(entry, &mut visited));

        for entry in walker.filter_map(std::result::Result::ok) {
            if Instant::now() >= deadline || hits.len() >= CANDIDATE_LIMIT {
                stopped_early = true;
                break;
            }
            if !entry.file_type().is_file() {
                continue;
            }
            let Some(name) = entry.file_name().to_str() else {
                continue;
            };
            let Some(score) = match_score(name, tokens, extension) else {
                continue;
            };
            let Ok(meta) = entry.metadata() else {
                continue;
            };

            let modified_at = meta.modified().unwrap_or(SystemTime::UNIX_EPOCH);
            hits.push(Hit {
                name: name.to_string(),
                path: display_path(&entry.path().to_path_buf()),
                size: human_size(meta.len()),
                size_bytes: meta.len(),
                modified: format_time(modified_at),
                score,
                modified_at,
            });
        }

        if stopped_early {
            break;
        }
    }

    // Сначала точность совпадения имени, при равенстве — более свежий файл.
    hits.sort_by(|a, b| {
        b.score
            .cmp(&a.score)
            .then_with(|| b.modified_at.cmp(&a.modified_at))
    });
    hits.truncate(limit);

    (hits, stopped_early)
}

fn keep_dir(entry: &DirEntry, visited: &mut HashSet<PathBuf>) -> bool {
    if !entry.file_type().is_dir() {
        return true;
    }

    let name = entry.file_name().to_string_lossy().to_lowercase();
    if name.starts_with('.') || SKIP_DIRS.contains(&name.as_str()) {
        return false;
    }

    // Домашняя папка перекрывает Рабочий стол и Документы — второй раз не идём.
    visited.insert(entry.path().to_path_buf())
}

/// Совпадение по основе слова, иначе «практика» не найдёт «практике».
fn token_matches(haystack: &str, token: &str) -> bool {
    if haystack.contains(token) {
        return true;
    }

    let chars: Vec<char> = token.chars().collect();
    if chars.len() < 5 {
        return false;
    }
    let stem: String = chars[..chars.len() - 2].iter().collect();
    haystack.contains(&stem)
}

/// Отделяет расширение от имени: человек говорит «ticket.pdf», а файл на диске
/// называется «tickets.pdf» — искать «.pdf» как часть имени нельзя.
fn split_query(query: &str) -> (Vec<String>, Option<String>) {
    let mut tokens: Vec<String> = query
        .to_lowercase()
        .split_whitespace()
        .map(str::to_string)
        .collect();

    let Some(last) = tokens.last().cloned() else {
        return (tokens, None);
    };
    let Some((stem, extension)) = last.rsplit_once('.') else {
        return (tokens, None);
    };

    let looks_like_extension = !stem.is_empty()
        && (1..=5).contains(&extension.chars().count())
        && extension.chars().all(char::is_alphanumeric);
    if !looks_like_extension {
        return (tokens, None);
    }

    *tokens.last_mut().expect("список не пуст") = stem.to_string();
    (tokens, Some(extension.to_string()))
}

/// Чем ближе имя к запросу целиком, тем выше балл; None — совпадения нет.
fn match_score(name: &str, tokens: &[String], extension: Option<&str>) -> Option<u8> {
    let lower = name.to_lowercase();

    if let Some(extension) = extension
        && !lower.ends_with(&format!(".{extension}"))
    {
        return None;
    }
    if !tokens.iter().all(|token| token_matches(&lower, token)) {
        return None;
    }

    let stem = lower.rsplit_once('.').map_or(lower.as_str(), |(stem, _)| stem);
    let joined = tokens.join(" ");

    Some(if stem == joined {
        3
    } else if lower.starts_with(&joined) {
        2
    } else {
        1
    })
}

fn human_size(bytes: u64) -> String {
    const UNITS: [&str; 5] = ["Б", "КБ", "МБ", "ГБ", "ТБ"];
    let mut size = bytes as f64;
    let mut unit = 0;
    while size >= 1024.0 && unit < UNITS.len() - 1 {
        size /= 1024.0;
        unit += 1;
    }
    if unit == 0 {
        format!("{bytes} Б")
    } else {
        format!("{size:.1} {}", UNITS[unit])
    }
}

fn format_time(time: SystemTime) -> String {
    chrono::DateTime::<chrono::Local>::from(time)
        .format("%d.%m.%Y %H:%M")
        .to_string()
}

fn display_path(path: &PathBuf) -> String {
    path.to_string_lossy().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ranks_exact_name_above_substring() {
        let tokens = vec!["отчёт".to_string()];
        assert_eq!(match_score("отчёт.docx", &tokens, None), Some(3));
        assert_eq!(match_score("отчёт по практике.docx", &tokens, None), Some(2));
        assert_eq!(match_score("мой отчёт.docx", &tokens, None), Some(1));
        assert_eq!(match_score("смета.xlsx", &tokens, None), None);
    }

    #[test]
    fn requires_every_token_but_tolerates_endings() {
        let tokens = vec!["отчёт".to_string(), "практика".to_string()];
        assert_eq!(match_score("Отчёт по практике.docx", &tokens, None), Some(1));
        assert_eq!(match_score("Отчёт по учёбе.docx", &tokens, None), None);
    }

    #[test]
    fn extension_is_matched_separately_from_the_name() {
        let (tokens, extension) = split_query("ticket.pdf");
        assert_eq!(tokens, vec!["ticket".to_string()]);
        assert_eq!(extension.as_deref(), Some("pdf"));

        // Файл называется чуть иначе, чем помнит человек, — всё равно находим.
        assert_eq!(match_score("tickets.pdf", &tokens, Some("pdf")), Some(2));
        assert_eq!(match_score("ticket.pdf", &tokens, Some("pdf")), Some(3));
        // А вот другое расширение отбрасываем.
        assert_eq!(match_score("ticket.docx", &tokens, Some("pdf")), None);
    }

    #[test]
    fn keeps_dotted_names_that_are_not_extensions() {
        let (tokens, extension) = split_query("версия 2.0 проекта");
        assert_eq!(extension, None);
        assert_eq!(tokens.len(), 3);
    }

    #[test]
    fn scales_size_units() {
        assert_eq!(human_size(512), "512 Б");
        assert_eq!(human_size(2048), "2.0 КБ");
        assert_eq!(human_size(5 * 1024 * 1024), "5.0 МБ");
    }
}
