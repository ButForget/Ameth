use serde::{Deserialize, Deserializer, Serialize, de::Error as DeError};
use std::collections::BTreeMap;
use std::fs;
use std::num::NonZeroU32;
use std::path::Path;

pub const AMETH_TOML_FILE_NAME: &str = "Ameth.toml";

#[derive(Debug, Default)]
pub struct AmethConfig {
    editor: Option<EditorConfig>,
    extra: BTreeMap<String, toml::Value>,
    ideas: IdeasConfig,
}

impl AmethConfig {
    pub fn load_or_default(path: &Path) -> Result<Self, String> {
        if !path.exists() {
            return Ok(Self::default());
        }

        if !path.is_file() {
            return Err(format!("invalid Ameth config path: {}", path.display()));
        }

        let content = fs::read_to_string(path)
            .map_err(|error| format!("failed to read {}: {error}", path.display()))?;
        let table = toml::from_str::<toml::Table>(&content)
            .map_err(|error| format!("failed to parse {}: {error}", path.display()))?;

        Self::from_table(table, path)
    }

    pub fn save(&self, path: &Path) -> Result<(), String> {
        let table = self.to_table(path)?;
        let content = toml::to_string(&table)
            .map_err(|error| format!("failed to serialize {}: {error}", path.display()))?;

        fs::write(path, content)
            .map_err(|error| format!("failed to write {}: {error}", path.display()))
    }

    pub fn set_value(&mut self, key: &str, value: toml::Value, path: &Path) -> Result<(), String> {
        let mut table = self.to_table(path)?;
        set_table_value(&mut table, key, value)?;
        *self = Self::from_table(table, path)?;

        Ok(())
    }

    fn from_table(mut table: toml::Table, path: &Path) -> Result<Self, String> {
        let editor = match table.remove("editor") {
            None => None,
            Some(value) => Some(EditorConfig::from_toml(value, path)?),
        };

        let ideas = match table.remove("ideas") {
            None => IdeasConfig::default(),
            Some(toml::Value::Table(ideas_table)) => {
                IdeasConfig::deserialize(toml::Value::Table(ideas_table))
                    .map_err(|error| format!("failed to parse {}: {error}", path.display()))?
            }
            Some(_) => {
                return Err(format!("invalid [ideas] table in {}", path.display()));
            }
        };

        Ok(Self {
            editor,
            extra: table.into_iter().collect(),
            ideas,
        })
    }

    fn to_table(&self, path: &Path) -> Result<toml::Table, String> {
        let mut table = toml::Table::new();

        for (key, value) in &self.extra {
            table.insert(key.clone(), value.clone());
        }

        if let Some(editor) = &self.editor {
            table.insert("editor".to_string(), editor.to_toml_value());
        }

        let ideas_value = toml::Value::try_from(&self.ideas)
            .map_err(|error| format!("failed to serialize {}: {error}", path.display()))?;
        let toml::Value::Table(ideas_table) = ideas_value else {
            unreachable!("ideas config always serializes as a table");
        };
        table.insert("ideas".to_string(), toml::Value::Table(ideas_table));

        Ok(table)
    }

    pub fn pinned_id(&self) -> Option<u32> {
        self.ideas.pinned.map(NonZeroU32::get)
    }

    pub fn set_pinned_id(&mut self, id: u32) {
        self.ideas.pinned = Some(
            NonZeroU32::new(id).expect("idea ids are validated before being written to config"),
        );
    }

    pub fn editor_command(&self) -> Option<(&str, &[String])> {
        let editor = self.editor.as_ref()?;
        Some((editor.program(), editor.args()))
    }
}

pub fn parse_config_value(raw: &str) -> toml::Value {
    toml::from_str::<toml::Table>(&format!("value = {raw}"))
        .ok()
        .and_then(|mut table| table.remove("value"))
        .unwrap_or_else(|| toml::Value::String(raw.to_string()))
}

#[derive(Debug, Eq, PartialEq)]
struct EditorConfig {
    parts: Vec<String>,
}

impl EditorConfig {
    fn from_toml(value: toml::Value, path: &Path) -> Result<Self, String> {
        let mut parts = match value {
            toml::Value::String(command) => vec![command.trim().to_string()],
            toml::Value::Array(values) => {
                let mut parts = Vec::new();

                for value in values {
                    let toml::Value::String(part) = value else {
                        return Err(format!(
                            "invalid root-level `editor` in {}; expected a string or array of strings",
                            path.display()
                        ));
                    };

                    parts.push(part);
                }

                parts
            }
            _ => {
                return Err(format!(
                    "invalid root-level `editor` in {}; expected a string or array of strings",
                    path.display()
                ));
            }
        };

        if parts.is_empty() || parts[0].trim().is_empty() {
            return Err(format!(
                "invalid root-level `editor` in {}; editor command cannot be empty",
                path.display()
            ));
        }

        parts[0] = parts[0].trim().to_string();

        Ok(Self { parts })
    }

    fn to_toml_value(&self) -> toml::Value {
        if self.parts.len() == 1 {
            return toml::Value::String(self.parts[0].clone());
        }

        toml::Value::Array(
            self.parts
                .iter()
                .map(|part| toml::Value::String(part.clone()))
                .collect(),
        )
    }

    fn program(&self) -> &str {
        &self.parts[0]
    }

    fn args(&self) -> &[String] {
        &self.parts[1..]
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
struct IdeasConfig {
    #[serde(default, deserialize_with = "deserialize_optional_pinned")]
    pinned: Option<NonZeroU32>,
    #[serde(flatten)]
    extra: BTreeMap<String, toml::Value>,
}

fn deserialize_optional_pinned<'de, D>(deserializer: D) -> Result<Option<NonZeroU32>, D::Error>
where
    D: Deserializer<'de>,
{
    let value = Option::<toml::Value>::deserialize(deserializer)?;

    match value {
        None => Ok(None),
        Some(toml::Value::Integer(raw)) if raw > 0 && raw <= i64::from(u32::MAX) => {
            Ok(Some(NonZeroU32::new(raw as u32).unwrap()))
        }
        Some(_) => Err(D::Error::custom("invalid ideas.pinned value")),
    }
}

fn set_table_value(table: &mut toml::Table, key: &str, value: toml::Value) -> Result<(), String> {
    let parts = parse_key_parts(key)?;
    let (last, parents) = parts
        .split_last()
        .expect("config keys must contain at least one segment");
    let mut current = table;

    for part in parents {
        if !current.contains_key(*part) {
            current.insert((*part).to_string(), toml::Value::Table(toml::Table::new()));
        }

        let next = current
            .get_mut(*part)
            .expect("config keys remain available while descending");
        let Some(next_table) = next.as_table_mut() else {
            return Err(format!("cannot set `{key}`: `{part}` is not a table"));
        };

        current = next_table;
    }

    current.insert((*last).to_string(), value);
    Ok(())
}

fn parse_key_parts(key: &str) -> Result<Vec<&str>, String> {
    let parts = key.split('.').collect::<Vec<_>>();

    if parts.is_empty() || parts.iter().any(|part| part.trim().is_empty()) {
        return Err("config key must use non-empty dot-separated segments".to_string());
    }

    Ok(parts)
}
