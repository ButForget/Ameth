use serde::{Deserialize, Deserializer, Serialize, de::Error as DeError};
use std::collections::BTreeMap;
use std::fs;
use std::num::NonZeroU32;
use std::path::Path;

pub const AMETH_TOML_FILE_NAME: &str = "Ameth.toml";

#[derive(Debug, Default, Serialize)]
pub struct AmethConfig {
    #[serde(flatten)]
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
        let mut table = toml::from_str::<toml::Table>(&content)
            .map_err(|error| format!("failed to parse {}: {error}", path.display()))?;

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
            extra: table.into_iter().collect(),
            ideas,
        })
    }

    pub fn save(&self, path: &Path) -> Result<(), String> {
        let mut table = toml::Table::new();

        for (key, value) in &self.extra {
            table.insert(key.clone(), value.clone());
        }

        let ideas_value = toml::Value::try_from(&self.ideas)
            .map_err(|error| format!("failed to serialize {}: {error}", path.display()))?;
        let toml::Value::Table(ideas_table) = ideas_value else {
            unreachable!("ideas config always serializes as a table");
        };
        table.insert("ideas".to_string(), toml::Value::Table(ideas_table));

        let content = toml::to_string(&table)
            .map_err(|error| format!("failed to serialize {}: {error}", path.display()))?;

        fs::write(path, content)
            .map_err(|error| format!("failed to write {}: {error}", path.display()))
    }

    pub fn pinned_id(&self) -> Option<u32> {
        self.ideas.pinned.map(NonZeroU32::get)
    }

    pub fn set_pinned_id(&mut self, id: u32) {
        self.ideas.pinned = Some(
            NonZeroU32::new(id).expect("idea ids are validated before being written to config"),
        );
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
