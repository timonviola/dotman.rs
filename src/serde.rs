use serde::de::{self, Deserializer, Visitor};
use serde::Deserialize;

use anyhow::Result;
use home;
use log::debug;
use std::collections::HashMap;
use std::fmt;
use std::fs;
use std::os::unix::fs::symlink;
use std::path::PathBuf;

pub const HOME_SYMBOL: char = '~';

/// expand the HOME_SYMBOL character containing path into absolute path
fn expand_home_path(path: &String) -> std::path::PathBuf {
    let binding = home::home_dir().unwrap();
    let home_path = binding.to_str().unwrap();
    // here we are confident that HOME_SYMBOL is present, as
    // it was checked before. It's ok to panic if that's not the case
    return std::path::PathBuf::from(&path.replace(HOME_SYMBOL, home_path));
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MyPath {
    path: std::path::PathBuf,
}

impl MyPath {
    pub fn capacity(&self) -> usize {
        self.path.capacity()
    }
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

#[derive(Debug, Deserialize)]
pub struct Outer {
    pub tool: HashMap<String, Tool>,
}

impl Outer {
    /// return the max format width
    pub fn max(&self) -> usize {
        let mut _i = 0usize;
        for (_, val) in &self.tool {
            _i = if _i < val.source.capacity() {
                val.source.capacity()
            } else {
                _i
            };
        }
        return _i;
    }
}

/// Check if file exists
pub fn is_valid_path(path: &MyPath) -> String {
    //let target = std::path::Path::new(&value);
    let out_color = match path.exists() {
        true => "Ok",
        _ => ":(",
    };
    String::from(out_color)
}

#[derive(Debug, Deserialize)]
pub struct Tool {
    pub source: MyPath,
    pub target: MyPath,
    pub tag: Option<String>,
}

impl Tool {
    /// convenience method around `path.exists()`
    pub fn validate(&self) -> bool {
        self.target.exists() && self.source.exists()
    }
    /// convenience method around `fs::symlink`
    pub fn create_link(&self) -> Result<(), Box<dyn std::error::Error>> {
        symlink(&self.source.path, &self.target.path)?;
        debug!(
            "created link: {:#?} -> {:#?}",
            &self.source.path, &self.target.path
        );
        Ok(())
    }

    /// get formatted, table like representation
    fn get_fromatted(&self, width: &usize) -> String {
        let _tag = match &self.tag {
            None => &String::from(""),
            Some(val) => val,
        };
        format!(
            "{: <8} {:.<width$} {:>3} -> {:.<40} {:.>3}",
            format!("{}", _tag),
            format!("{}", self.source),
            format!("{}", is_valid_path(&self.source)),
            format!("{}", self.target),
            format!("{}", is_valid_path(&self.target)),
            width = width
        )
    }
}
/// Pretty printing for config entries
impl fmt::Display for Tool {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{:.<68} {:>3} -> {:.<40} {:.>3}",
            format!("{}", self.source),
            format!("{}", is_valid_path(&self.source)),
            format!("{}", self.target),
            format!("{}", is_valid_path(&self.target))
        )
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
