use crate::config::{AMETH_TOML_FILE_NAME, AmethConfig, parse_config_value};
use clap::Args;
use std::env;
use std::path::PathBuf;

#[derive(Args, Debug)]
#[command(
    about = "Set a value in Ameth.toml",
    override_usage = "ameth config <KEY> <VALUE>",
    after_help = "Examples:\n  ameth config editor nvim\n  ameth config editor '[\"code\", \"--wait\"]'\n  ameth config ideas.pinned 4"
)]
pub struct ConfigArgs {
    #[arg(value_name = "KEY")]
    key: String,

    #[arg(value_name = "VALUE")]
    value: String,
}

pub fn run(args: ConfigArgs) -> Result<(), String> {
    let config_path = project_config_path()?;
    let mut config = AmethConfig::load_or_default(&config_path)?;

    config.set_value(&args.key, parse_config_value(&args.value), &config_path)?;
    config.save(&config_path)?;

    println!("Updated {}: {}", AMETH_TOML_FILE_NAME, args.key);
    Ok(())
}

fn project_config_path() -> Result<PathBuf, String> {
    let root =
        env::current_dir().map_err(|error| format!("failed to read current directory: {error}"))?;
    let config_path = root.join(AMETH_TOML_FILE_NAME);

    if !config_path.is_file() {
        return Err("current directory is not an Ameth project".to_string());
    }

    Ok(config_path)
}
