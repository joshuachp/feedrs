use clap::{crate_description, crate_name, crate_version, App, Arg};
use serde::Deserialize;
use std::env;
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::{Path, PathBuf};

// Configuration structure for serialization and deserialization
#[derive(Deserialize, Debug, Eq, PartialEq)]
pub struct ConfigFile {
    // NOTE: This could be a Url and we could check the validity of our config
    sources: Option<Vec<String>>,
}

// Configuration structure for use in the application
#[derive(Debug, Eq, PartialEq, Clone)]
pub struct Config {
    config_path: PathBuf,
    sources: Vec<String>,
}

/**
 * Get the path of the config file. It will use the path passed as an argument or the environment
 * variable XDG_CONFIG_HOME. If neither of those are provided it will use the path:
 * `~/config/feedrs/feedrs.toml`
 */
fn get_config_path(path_arg: Option<&str>) -> PathBuf {
    let path;
    // Check if file exists only for the provided path
    if let Some(path_arg) = path_arg {
        path = PathBuf::from(path_arg);
        if !path.is_file() {
            panic!("{}: File doesn't exists", path_arg);
        }
    } else {
        let xdg_config = env::var("XDG_CONFIG_HOME").unwrap_or(String::from("~/.config"));
        path = PathBuf::from(format!("{}/feedrs/feedrs.toml", xdg_config,));
    }
    path
}

fn read_config_file(path: &Path) -> std::io::Result<ConfigFile> {
    let file = File::open(path)?;
    let mut buf_reader = BufReader::new(file);
    let mut contents = String::new();
    buf_reader.read_to_string(&mut contents)?;
    match toml::from_str(&contents) {
        Ok(config_file) => Ok(config_file),
        Err(err) => {
            panic!("{}", err);
        }
    }
}

// Return a configuration istance
pub fn config() -> Config {
    let matches = App::new(crate_name!())
        .version(crate_version!())
        .about(crate_description!())
        .arg(
            Arg::with_name("config")
                .short("c")
                .long("config")
                .value_name("CONFIG_PATH")
                .help("Path to the Toml config file")
                .takes_value(true),
        )
        .get_matches();

    let config_path = get_config_path(matches.value_of("config"));

    let config_file = match read_config_file(&config_path) {
        Err(err) => {
            panic!("{}", err);
        }
        Ok(res) => res,
    };

    Config {
        config_path,
        sources: config_file.sources.unwrap_or_else(|| vec![]),
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_get_config_path_default() {
        env::remove_var("XDG_CONFIG_HOME");
        env::var("XDG_CONFIG_HOME").unwrap_err();
        assert_eq!(
            get_config_path(None),
            PathBuf::from("~/.config/feedrs/feedrs.toml")
        );
    }

    #[test]
    fn test_get_config_path_env() {
        env::set_var("XDG_CONFIG_HOME", "tests");
        env::var("XDG_CONFIG_HOME").unwrap();
        assert_eq!(
            get_config_path(None),
            PathBuf::from("tests/feedrs/feedrs.toml")
        );
    }

    #[test]
    fn test_get_config_path_arg() {
        env::remove_var("XDG_CONFIG_HOME");
        env::var("XDG_CONFIG_HOME").unwrap_err();
        assert_eq!(
            get_config_path(Some("tests/feedrs/feedrs.toml")),
            PathBuf::from("tests/feedrs/feedrs.toml")
        );
    }

    #[test]
    #[should_panic]
    fn test_get_config_path_arg_panic() {
        env::remove_var("XDG_CONFIG_HOME");
        env::var("XDG_CONFIG_HOME").unwrap();
        get_config_path(Some("does/not/exists.toml"));
    }

    #[test]
    fn test_read_config_file() {
        let conf = ConfigFile {
            sources: Some(
                ["source_1", "source_2", "source_3"]
                    .iter()
                    .map(|x| String::from(*x))
                    .collect(),
            ),
        };
        let config_file = read_config_file(Path::new("tests/feedrs/feedrs.toml"));
        assert!(config_file.is_ok());
        assert_eq!(config_file.unwrap(), conf);
    }

    #[test]
    #[should_panic]
    fn test_read_config_file_panic() {
        let _ = read_config_file(Path::new("tests/test_err.toml"));
    }

    #[test]
    fn test_read_config_file_err() {
        let config_file = read_config_file(Path::new("does/not/exists.toml"));
        assert!(config_file.is_err());
    }

    #[test]
    fn test_config() {
        env::set_var("XDG_CONFIG_HOME", "tests");
        let config = config();
        let expected = Config {
            config_path: PathBuf::from("tests/feedrs/feedrs.toml"),
            sources: ["source_1", "source_2", "source_3"]
                .iter()
                .map(|x| String::from(*x))
                .collect(),
        };
        assert_eq!(config, expected);
    }

    #[test]
    #[should_panic]
    fn test_config_panics() {
        env::set_var("XDG_CONFIG_HOME", "does/not/exists");
        dbg!(env::var("XDG_CONFIG_HOME").unwrap());
        let config = config();
        dbg!(config.clone());
        let expected = Config {
            config_path: PathBuf::from("tests/feedrs/feedrs.toml"),
            sources: ["source_1", "source_2", "source_3"]
                .iter()
                .map(|x| String::from(*x))
                .collect(),
        };
        assert_eq!(config, expected);
    }
}
