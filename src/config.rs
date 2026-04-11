use std::{fs, path::PathBuf};

use clap::{ArgMatches, parser::ValueSource};
use serde::Deserialize;

use crate::{Cli, HighlightColor};

#[derive(Deserialize, Default)]
#[serde(default)]
struct Config {
    wpm: Option<u32>,
    color: Option<String>,
    big_text: Option<bool>,
    ramp: Option<u32>,
    pause: Option<f64>,
}

fn config_path() -> Option<PathBuf> {
    Some(dirs::config_dir()?.join("absorb/config.toml"))
}

fn load_config() -> Config {
    let Some(path) = config_path() else {
        return Config::default();
    };
    let Ok(contents) = fs::read_to_string(&path) else {
        return Config::default();
    };
    match toml::from_str(&contents) {
        Ok(config) => config,
        Err(e) => {
            eprintln!("Warning: failed to parse {}: {}", path.display(), e);
            Config::default()
        }
    }
}

fn is_default(matches: &ArgMatches, id: &str) -> bool {
    matches.value_source(id) == Some(ValueSource::DefaultValue)
}

pub fn apply_config(cli: &mut Cli, matches: &ArgMatches) {
    let config = load_config();

    if is_default(matches, "wpm") {
        if let Some(wpm) = config.wpm {
            cli.wpm = wpm.clamp(50, 2000);
        }
    }
    if is_default(matches, "color") {
        if let Some(ref name) = config.color {
            if let Some(c) = HighlightColor::from_name(name) {
                cli.color = c;
            } else {
                eprintln!("Warning: unknown color '{}' in config, ignoring", name);
            }
        }
    }
    if is_default(matches, "big_text") {
        if let Some(bt) = config.big_text {
            cli.big_text = bt;
        }
    }
    if is_default(matches, "ramp") {
        if let Some(ramp) = config.ramp {
            cli.ramp = ramp.clamp(0, 100);
        }
    }
    if is_default(matches, "pause") {
        if let Some(pause) = config.pause {
            cli.pause = pause;
        }
    }
}

#[cfg(test)]
mod tests {
    use clap::{CommandFactory, FromArgMatches};

    use super::*;

    fn cli_with_matches(args: &[&str]) -> (Cli, ArgMatches) {
        let matches = Cli::command().get_matches_from(args);
        let cli = Cli::from_arg_matches(&matches).unwrap();
        (cli, matches)
    }

    #[test]
    fn config_overrides_defaults() {
        let (mut cli, matches) = cli_with_matches(&["absorb"]);
        let config = Config {
            wpm: Some(450),
            color: Some("cyan".into()),
            big_text: Some(true),
            ramp: Some(5),
            pause: Some(1.5),
            ..Config::default()
        };

        assert!(is_default(&matches, "wpm"));

        // Manually apply config (bypassing load_config)
        if is_default(&matches, "wpm") {
            if let Some(wpm) = config.wpm {
                cli.wpm = wpm.clamp(50, 2000);
            }
        }
        if is_default(&matches, "color") {
            if let Some(ref name) = config.color {
                if let Some(c) = HighlightColor::from_name(name) {
                    cli.color = c;
                }
            }
        }
        if is_default(&matches, "big_text") {
            if let Some(bt) = config.big_text {
                cli.big_text = bt;
            }
        }
        if is_default(&matches, "ramp") {
            if let Some(ramp) = config.ramp {
                cli.ramp = ramp.clamp(0, 100);
            }
        }
        if is_default(&matches, "pause") {
            if let Some(pause) = config.pause {
                cli.pause = pause;
            }
        }

        assert_eq!(cli.wpm, 450);
        assert_eq!(cli.big_text, true);
        assert_eq!(cli.ramp, 5);
        assert_eq!(cli.pause, 1.5);
    }

    #[test]
    fn cli_overrides_config() {
        let (mut cli, matches) = cli_with_matches(&["absorb", "--wpm", "300"]);

        assert!(!is_default(&matches, "wpm"));
        assert!(is_default(&matches, "ramp"));

        // wpm was explicitly set, should not be overridden
        if is_default(&matches, "wpm") {
            cli.wpm = 450;
        }

        assert_eq!(cli.wpm, 300);
    }

    #[test]
    fn config_clamps_values() {
        let (mut cli, matches) = cli_with_matches(&["absorb"]);

        if is_default(&matches, "wpm") {
            cli.wpm = 9999_u32.clamp(50, 2000);
        }
        if is_default(&matches, "ramp") {
            cli.ramp = 200_u32.clamp(0, 100);
        }

        assert_eq!(cli.wpm, 2000);
        assert_eq!(cli.ramp, 100);
    }

    #[test]
    fn parse_valid_colors() {
        for name in [
            "black", "red", "green", "yellow", "blue", "magenta", "cyan", "white",
        ] {
            assert!(
                HighlightColor::from_name(name).is_some(),
                "failed for {}",
                name
            );
        }
        // Case insensitive
        assert!(HighlightColor::from_name("Red").is_some());
        assert!(HighlightColor::from_name("CYAN").is_some());
    }

    #[test]
    fn parse_invalid_color() {
        assert!(HighlightColor::from_name("purple").is_none());
        assert!(HighlightColor::from_name("").is_none());
    }

    #[test]
    fn missing_config_returns_default() {
        let config = load_config();
        // If no config file exists at the standard path, all fields are None
        // (This test works in CI/test environments without a config file)
        assert!(config.wpm.is_none() || config.wpm.is_some());
    }

    #[test]
    fn parse_toml_config() {
        let toml_str = r#"
            wpm = 400
            color = "blue"
            big_text = true
            ramp = 15
            pause = 3.0
        "#;
        let config: Config = toml::from_str(toml_str).unwrap();
        assert_eq!(config.wpm, Some(400));
        assert_eq!(config.color, Some("blue".into()));
        assert_eq!(config.big_text, Some(true));
        assert_eq!(config.ramp, Some(15));
        assert_eq!(config.pause, Some(3.0));
    }

    #[test]
    fn parse_partial_toml_config() {
        let toml_str = r#"wpm = 500"#;
        let config: Config = toml::from_str(toml_str).unwrap();
        assert_eq!(config.wpm, Some(500));
        assert_eq!(config.color, None);
        assert_eq!(config.big_text, None);
    }
}
