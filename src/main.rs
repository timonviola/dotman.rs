use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use clap_verbosity_flag::{Verbosity, InfoLevel};
use serde::de::{self, Deserializer, Visitor};
use serde::Deserialize;
use std::collections::HashMap;
use std::path::PathBuf;
//use error::{Error, Result};
use colored::*;
use home;
use std::fmt;
use std::fs;
use std::os::unix::fs::symlink;
use toml;
use log::{info, debug};

const HOME_SYMBOL: char = '~';

/// expand the HOME_SYMBOL character containing path into absolute path
fn expand_home_path(path: &String) -> std::path::PathBuf {
    let binding = home::home_dir().unwrap();
    let home_path = binding.to_str().unwrap();
    // here we are confident that HOME_SYMBOL is present, as
    // it was checked before. It's ok to panic if that's not the case
    return std::path::PathBuf::from(&path.replace(HOME_SYMBOL, home_path));
}

/// get default config file location which is $HOME/.dotman.toml
fn default_config_file() -> std::path::PathBuf {
    let mut path = home::home_dir().unwrap();
    path.push(".dotman.toml");
    return path;
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct MyPath {
    path: std::path::PathBuf,
}

impl<'de> Deserialize<'de> for MyPath {
    fn deserialize<D>(deserializer: D) -> Result<MyPath, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct MyPathVisitor;

        impl<'de> Visitor<'de> for MyPathVisitor {
            type Value = MyPath;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("struct MyPath")
            }

            fn visit_str<E>(self, value: &str) -> Result<MyPath, E>
            where
                E: de::Error,
            {
                Ok(MyPath::new(&value.to_owned()))
            }
        }
        deserializer.deserialize_string(MyPathVisitor)
    }
}

impl MyPath {
    pub fn new(path: &String) -> MyPath {
        let pt = match path.contains(HOME_SYMBOL) {
            true => expand_home_path(path),
            false => PathBuf::from(path),
        };
        return MyPath { path: pt };
    }
    pub fn exists(&self) -> bool {
        self.path.exists()
    }
    pub fn remove_file(&self) {
        // TODO return Result
        let _ = fs::remove_file(&self.path);
    }
}

impl fmt::Display for MyPath {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self.path)
    }
}

#[derive(Parser)]
#[command(author, version, about, long_about=None, propagate_version = true)]
struct Cli {
    // config file
    #[arg(short,
          long,
          value_name = "FILE",
          default_value = default_config_file().into_os_string())]
    file: std::path::PathBuf,

    // log-level
    #[command(flatten)]
    verbose: Verbosity<InfoLevel>,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// creates symlinks to file
    Link {},
    /// destroys symlinks
    Purge {},
    /// show status of links compared to config files
    Show {},
}

#[derive(Debug, Deserialize)]
struct Outer {
    tool: HashMap<String, Tool>,
}

#[derive(Debug, Deserialize)]
struct Tool {
    source: MyPath,
    target: MyPath,
}

impl Tool {
    /// convinience method around `path.exists()`
    fn validate(&self) -> bool {
        self.target.exists() && self.source.exists()
    }
    /// convinience method around `fs::symlink`
    fn create_link(&self) -> Result<(), Box<dyn std::error::Error>> {
        symlink(&self.source.path, &self.target.path)?;
        debug!("created link: {:#?} -> {:#?}",&self.source.path, &self.target.path); 
        Ok(())
    }
}

/// Pretty printing for config entries
impl fmt::Display for Tool {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{} {} -> {} {}",
            format!("{:44}", self.source),
            format!("{:3}", is_valid_path(&self.source)),
            self.target,
            format!("{:3}", is_valid_path(&self.target))
        )
    }
}

/// Check if file exists
fn is_valid_path(path: &MyPath) -> ColoredString {
    //let target = std::path::Path::new(&value);
    let out_color = match path.exists() {
        true => ("Ok", Color::Green),
        _ => (":(", Color::Red),
    };
    out_color.0.color(out_color.1).bold()
}

fn main() -> Result<()> {
    //
    // `parse` needs to be calle in main
    let args = Cli::parse();
    env_logger::Builder::new()
        .filter_level(args.verbose.log_level_filter())
        .init();

    let content = std::fs::read_to_string(&args.file)
        .with_context(|| format!("could not read file `{}`", args.file.display()))?;
    // TOML handling

    let config: Outer = toml::from_str(&content)?;

    match &args.command {
        Commands::Link {} => {
            info!("original dotman behaviour");
            link(&config);
        }
        Commands::Purge {} => {
            info!("destroying links");
            purge(&config);
        }
        Commands::Show {} => {
            info!("show links");
            show(&config);
        }
    }
    Ok(())
}

/// Destroy symlinks
fn purge(config: &Outer) {
    // for each key: validate and create link
    for (_key, val) in &config.tool {
        if val.target.exists() {
            val.target.remove_file();
            debug!("removed {:?}", val.target);
        }
    }
}

/// Create symlink for values
fn link(config: &Outer) {
    // for each key: validate and create link
    for (_key, val) in &config.tool {
        match val.validate() {
            true => info!("File exists {:#?}. Skipping.",_key),
            false => debug!("created link: {:#?} -> {:#?}",_key, val),
        }

        let _ = val.create_link();
    }
}

/// Display the content of each file *nicely*
fn show(config: &Outer) {
    for (key, val) in &config.tool {
        info!("[{key}]\n\t{val}");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_home_path_expand() {
        assert_eq!(
            home::home_dir().unwrap(),
            expand_home_path(&String::from("~"))
        );
    }
    #[test]
    fn test_deser_toml() {
        let mut expected = HashMap::new();
        expected.insert("path".to_owned(), MyPath::new(&String::from("/home/timon")));
        let toml = r#"path = "/home/timon""#;
        assert_eq!(
            expected,
            toml::from_str::<HashMap<String, MyPath>>(&toml).unwrap()
        );
    }
}

