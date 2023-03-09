use super::suggestion::{executable::ExecutableSuggestion, program::ProgramSuggestion};
use serde::{Deserialize, Serialize};
use std::env;
use std::fs;
use std::io::prelude::*;

lazy_static::lazy_static! {
    static ref CACHE_DIR: String = format!("{}/.cache/lanch/cachefile", env::var("HOME").unwrap());
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct LanchCache {
    // Programs are applications found in /usr/share/applications
    pub programs: Vec<ProgramSuggestion>,

    // files in $PATH
    pub executables: Vec<ExecutableSuggestion>,
}

impl LanchCache {
    fn generate_programs(icon_theme: Option<&str>) -> Result<Vec<ProgramSuggestion>, std::io::Error> {
        let mut ret: Vec<ProgramSuggestion> = Vec::new();

        for entry in fs::read_dir("/usr/share/applications")? {
            let file = fs::read_to_string(entry?.path())?;
            let mut fields: [&str; 3] = ["", "", ""];

            for line in file.lines() {
                if line.starts_with("Name=") {
                    let (_, val) = line.split_once('=').unwrap_or(("", ""));

                    if !val.is_empty() {
                        fields[0] = val;
                    }
                } else if line.starts_with("Exec=") {
                    let (_, val) = line.split_once('=').unwrap_or(("", ""));

                    if !val.is_empty() {
                        fields[1] = val;
                    }
                } else if line.starts_with("Icon=") {
                    let (_, val) = line.split_once('=').unwrap_or(("", ""));

                    if !val.is_empty() {
                        fields[2] = val;
                    }
                }

                if fields.iter().all(|e| !e.is_empty()) {
                    break;
                }
            }

            let icon_path = if let Some(theme) = icon_theme {
                freedesktop_icons::lookup(fields[2])
                    .with_size(48)
                    .with_theme(theme)
                    .find()
            } else {
                freedesktop_icons::lookup(fields[2])
                    .with_size(48)
                    .find()
            };

            ret.push(ProgramSuggestion::new(fields[0], fields[1], icon_path));
        }

        Ok(ret)
    }

    fn generate_executables() -> Result<Vec<ExecutableSuggestion>, std::io::Error> {
        let mut ret: Vec<ExecutableSuggestion> = Vec::new();
        let path = env::var("PATH").unwrap_or("/bin".to_string());

        for dir in path.split(':') {
            // filter out directories and invalid entries
            let dir = match fs::read_dir(dir) {
                Ok(d) => d,
                Err(_) => continue,
            };
            for entry in dir.flatten() {
                if !entry.metadata()?.is_dir() {
                    ret.push(ExecutableSuggestion::new(
                        entry.file_name().to_str().unwrap(),
                        entry.path().to_str().unwrap(),
                    ))
                }
            }
        }

        Ok(ret)
    }

    // Generates new cache and writes it to disk
    pub fn new(icon_theme: Option<&str>) -> Result<Self, Box<dyn std::error::Error>> {
        println!("[CACHE] generating new cache at {}", CACHE_DIR.clone());
        use std::fs::File;

        let cache = Self {
            programs: Self::generate_programs(icon_theme)?,
            executables: Self::generate_executables()?,
        };

        let mut cache_file = match File::open(CACHE_DIR.clone()) {
            Ok(f) => f,
            Err(_) => File::create(CACHE_DIR.clone()).unwrap(),
        };

        let encoded: Vec<u8> = bincode::serialize(&cache).unwrap();
        cache_file.write_all(&encoded).unwrap();

        Ok(cache)
    }

    pub fn from_disk_or_new(icon_theme: Option<&str>) -> Result<Self, Box<dyn std::error::Error>> {
        let data = fs::read(CACHE_DIR.clone());
        match data {
            Ok(data) => {
                let decoded: LanchCache = bincode::deserialize(&data[..])?;
                Ok(decoded)
            }
            Err(_) => Self::new(icon_theme),
        }
    }
}
