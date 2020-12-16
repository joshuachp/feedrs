use clap::{crate_description, crate_name, crate_version, value_t, App, Arg};
use serde::Deserialize;
use std::{
    collections::HashSet,
    env,
    ffi::OsString,
    fs::{create_dir_all, File},
    io::{self, BufReader, Read},
    path::{Path, PathBuf},
    sync::Arc,
};

// Configuration structure for serialization and deserialization
#[derive(Deserialize, Debug, Eq, PartialEq)]
struct ConfigFile {
    update_interval: Option<u64>,
    // NOTE: This could be a Url and we could check the validity of our config
    sources: Option<Vec<String>>,
}

// Configuration structure for use in the application
#[derive(Debug, Eq, PartialEq, Clone)]
pub struct Config {
    pub config_path: PathBuf,
    // We leve here the database uri since we can't use the pool directly
    pub cache_path: PathBuf,
    // Defaults to 5 minutes
    pub update_interval: u64,
    pub sources: HashSet<Arc<String>>,
}

/**
 * Get the path of the config file. It will use the path passed as an argument or the environment
 * variable XDG_CONFIG_HOME. If neither of those are provided it will use the path:
 * `~/config/feedrs/feedrs.toml`
 */
fn create_config_path(path_arg: Option<&str>) -> io::Result<PathBuf> {
    let path;
    // Check if file exists only for the provided path
    if let Some(path_arg) = path_arg {
        path = PathBuf::from(path_arg);
        if !path.is_file() {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                format!("Config file not found {}", path_arg),
            ));
        }
    } else {
        let mut xdg_config = env::var("XDG_CONFIG_HOME").unwrap_or_else(|_| {
            let mut config = env::var("HOME").unwrap();
            config.push_str("/.config");
            return config;
        });
        xdg_config.push_str("/feedrs/feedrs.toml");
        path = PathBuf::from(xdg_config);
        // Create file if doesn't exists
        if !path.is_file() {
            if let Some(parent) = path.parent() {
                create_dir_all(parent)?;
            }
            File::create(&path)?;
        }
    }
    Ok(path)
}

fn read_config_file(path: &Path) -> io::Result<ConfigFile> {
    let file = File::open(path)?;
    let mut buf_reader = BufReader::new(file);
    let mut contents = String::new();
    buf_reader.read_to_string(&mut contents)?;
    match toml::from_str(&contents) {
        Ok(config_file) => Ok(config_file),
        // Convert error from serde to io to pass it down
        Err(err) => Err(io::Error::new(io::ErrorKind::Other, err)),
    }
}

fn create_cache_path() -> io::Result<PathBuf> {
    let mut cache = env::var("HOME").unwrap();
    cache.push_str("/.cache/feedrs/cache.db");
    let path = PathBuf::from(&cache);
    if !path.is_file() {
        // We only create the folders here since the database pool is initialized afterwards
        if let Some(parent) = path.parent() {
            create_dir_all(parent)?;
        }
    }
    Ok(path)
}

// Return a configuration instance
pub fn config<I, T>(args: I) -> io::Result<Config>
where
    I: IntoIterator<Item = T>,
    T: Into<OsString> + Clone,
{
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
        .arg(
            Arg::with_name("update")
                .short("u")
                .long("update")
                .value_name("UPDATE_INTERVAL")
                .help("Interval of time in seconds to check the sources for updates")
                .takes_value(true),
        )
        .get_matches_from(args);

    let config_path = create_config_path(matches.value_of("config"))?;
    let config_file = read_config_file(&config_path)?;
    let cache_path = create_cache_path()?;
    let sources = config_file
        .sources
        .unwrap_or_else(|| vec![])
        .into_iter()
        .map(|x| Arc::new(x))
        .collect();
    let update_interval = value_t!(matches.value_of("update"), u64)
        .unwrap_or(config_file.update_interval.unwrap_or(300));

    Ok(Config {
        config_path,
        cache_path,
        update_interval,
        sources,
    })
}

#[cfg(test)]
mod test {

    use super::*;

    #[test]
    fn test_create_config_path() {
        env::remove_var("XDG_CONFIG_HOME");
        env::var("XDG_CONFIG_HOME").unwrap_err();
        let expected = PathBuf::from(format!(
            "{}/.config/feedrs/feedrs.toml",
            env::var("HOME").unwrap()
        ));
        assert_eq!(create_config_path(None).unwrap(), expected);
        assert!(expected.is_file())
    }

    #[test]
    fn test_create_config_path_env() {
        env::set_var("XDG_CONFIG_HOME", "tests");
        assert_eq!(env::var("XDG_CONFIG_HOME").unwrap(), "tests");
        assert_eq!(
            create_config_path(None).unwrap(),
            PathBuf::from("tests/feedrs/feedrs.toml")
        );
    }

    #[test]
    fn test_create_config_path_arg() {
        env::remove_var("XDG_CONFIG_HOME");
        env::var("XDG_CONFIG_HOME").unwrap_err();
        assert_eq!(
            create_config_path(Some("tests/feedrs/feedrs.toml")).unwrap(),
            PathBuf::from("tests/feedrs/feedrs.toml")
        );
    }

    #[test]
    fn test_create_config_path_arg_err() {
        env::remove_var("XDG_CONFIG_HOME");
        env::var("XDG_CONFIG_HOME").unwrap_err();
        create_config_path(Some("does/not/exists.toml")).unwrap_err();
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
            update_interval: Some(200),
        };
        let config_file = read_config_file(Path::new("tests/feedrs/feedrs.toml"));
        assert!(config_file.is_ok());
        assert_eq!(config_file.unwrap(), conf);
    }

    #[test]
    fn test_read_config_file_err_malformed() {
        let config_file = read_config_file(Path::new("tests/test_err.toml"));
        assert!(config_file.is_err());
    }

    #[test]
    fn test_read_config_file_err_not_exists() {
        let config_file = read_config_file(Path::new("does/not/exists.toml"));
        assert!(config_file.is_err());
    }

    #[test]
    fn test_config_file_arg() {
        let config = config(vec!["feedrs", "-c", "tests/feedrs/feedrs.toml"]).unwrap();
        let home = env::var("HOME").unwrap();
        let sources = ["source_1", "source_2", "source_3"]
            .iter()
            .map(|x| Arc::new(String::from(*x)))
            .collect();
        let expected = Config {
            config_path: PathBuf::from("tests/feedrs/feedrs.toml"),
            cache_path: PathBuf::from(format!("{}/.cache/feedrs/cache.db", home)),
            update_interval: 200,
            sources,
        };
        assert_eq!(config, expected);
    }
}
