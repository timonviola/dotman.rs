use std::collections::HashMap;

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use clap_verbosity_flag::{InfoLevel, Verbosity};
use home;
use log::{debug, info, warn};
use toml;

use dotman::format::{get_table, add_row};
use dotman::serde::{is_valid_path, Outer, Tool};

/// get default config file location which is $HOME/.dotman.toml
fn default_config_file() -> std::path::PathBuf {
    let mut path = home::home_dir().unwrap();
    path.push(".dotman.toml");
    return path;
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

    // tag to apply
    #[arg(short,
          long,
          help = "Use tags to apply links selectively",
          value_name = "TAG",
          default_value = None)]
    tag: Option<String>,

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

pub fn main() -> Result<()> {
    //
    // `parse` needs to be called in main
    let args = Cli::parse();
    env_logger::Builder::new()
        .filter_level(args.verbose.log_level_filter())
        .init();

    let content = std::fs::read_to_string(&args.file)
        .with_context(|| format!("could not read file `{}`", args.file.display()))?;
    
    // TOML handling
    let config = match args.tag {
        None => {
            match toml::from_str(&content) {
                Ok(content) => content,
                Err(error) => panic!("Problem reading the contents of the configuration file: {error:?}"),
            }
        }
        Some(ref t) => {
            let _config: Outer = toml::from_str(&content)?;
            let mut m: HashMap<String, Tool> = _config.tool;
            debug!("Retaining tag: {:?}", t);
            m.retain(|_k, _v| { 
                match &_v.tag {
                    None => false,
                    Some(tag) => tag.contains(t)
                }
            });
            let config: Outer = Outer { tool: m };
            config
        }
    };

    match &config.tool.is_empty() {
        true => {
            warn!("No configuration entries. Tags: {:?}\nExiting.", &args.tag);
            return Ok(());
        }
        false => (),
    };


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
            true => info!("File exists {:#?}. Skipping.", _key),
            false => debug!("created link: {:#?} -> {:#?}", _key, val),
        }

        let _ = val.create_link();
    }
}

/// Display the content of each file *nicely*
fn show(config: &Outer) {
    let mut table = get_table();
    let _width = config.max();
    for (key, val) in &config.tool {
        let _tag = match &val.tag {
            None => &String::from(""),
            Some(val) => val,
        };
        add_row(&mut table, 
                key,
                _tag,
                &is_valid_path(&val.target),
                &val.source,
        );
    }
    print!("\n");

    println!("{table}");
}

#[cfg(test)]
mod tests {
    use super::*;
    use dotman::serde::MyPath;
    use std::collections::HashMap;

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
