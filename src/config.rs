use std::{fs, path::PathBuf};

use clap::{ArgMatches, parser::ValueSource};

use crate::Cli;

fn config_path() -> Option<PathBuf> {
    Some(dirs::config_dir()?.join("absorb/config.toml"))
}

fn load_config() -> Cli {
    let Some(path) = config_path() else {
        return Cli::default();
    };
    let Ok(contents) = fs::read_to_string(&path) else {
        return Cli::default();
    };
    match toml::from_str(&contents) {
        Ok(config) => config,
        Err(e) => {
            eprintln!("Warning: failed to parse {}: {}", path.display(), e);
            Cli::default()
        }
    }
}

fn is_default(matches: &ArgMatches, id: &str) -> bool {
    matches.value_source(id) == Some(ValueSource::DefaultValue)
}

pub fn apply_config(cli: &mut Cli, matches: &ArgMatches) {
    let config = load_config();

    if is_default(matches, "wpm") {
        cli.wpm = config.wpm;
    }
    if is_default(matches, "color") {
        cli.color = config.color;
    }
    if is_default(matches, "big_text") {
        cli.big_text = config.big_text;
    }
    if is_default(matches, "ramp") {
        cli.ramp = config.ramp;
    }
    if is_default(matches, "pause") {
        cli.pause = config.pause;
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
        assert!(is_default(&matches, "wpm"));

        let config: Cli = toml::from_str(
            r#"
            wpm = 450
            color = "cyan"
            big_text = true
            ramp = 5
            pause = 1.5
        "#,
        )
        .unwrap();

        if is_default(&matches, "wpm") {
            cli.wpm = config.wpm;
        }
        if is_default(&matches, "color") {
            cli.color = config.color;
        }
        if is_default(&matches, "big_text") {
            cli.big_text = config.big_text;
        }
        if is_default(&matches, "ramp") {
            cli.ramp = config.ramp;
        }
        if is_default(&matches, "pause") {
            cli.pause = config.pause;
        }

        assert_eq!(cli.wpm, 450);
        assert_eq!(cli.big_text, true);
        assert_eq!(cli.ramp, 5);
        assert_eq!(cli.pause, 1.5);
    }

    #[test]
    fn cli_overrides_config() {
        let (cli, matches) = cli_with_matches(&["absorb", "--wpm", "300"]);

        assert!(!is_default(&matches, "wpm"));
        assert!(is_default(&matches, "ramp"));
        assert_eq!(cli.wpm, 300);
    }

    #[test]
    fn parse_toml_config() {
        let config: Cli = toml::from_str(
            r#"
            wpm = 400
            color = "blue"
            big_text = true
            ramp = 15
            pause = 3.0
        "#,
        )
        .unwrap();

        assert_eq!(config.wpm, 400);
        assert_eq!(config.big_text, true);
        assert_eq!(config.ramp, 15);
        assert_eq!(config.pause, 3.0);
    }

    #[test]
    fn parse_partial_toml_config() {
        let config: Cli = toml::from_str(r#"wpm = 500"#).unwrap();
        assert_eq!(config.wpm, 500);
        // Other fields should be defaults
        assert_eq!(config.ramp, 10);
        assert_eq!(config.pause, 2.0);
    }
}
