use super::suggestion::executable::{ExecutableSuggestion, ProgramSuggestion};
use serde::{Deserialize, Serialize};
use std::env;
use std::fs::{self, File};
use std::io::prelude::*;
use std::path::PathBuf;

use std::rc::Rc;

lazy_static::lazy_static! {
    static ref CACHE_FILE_PATH: PathBuf = PathBuf::from(format!("{}/.cache/lanch/cachefile", env::var("HOME").unwrap()));
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct LanchCache {
    // Programs are applications found in /usr/share/applications
    pub programs: Vec<ProgramSuggestion>,

    // files in $PATH
    pub executables: Vec<ExecutableSuggestion>,
}

pub struct LanchCacheRc {
    // Programs are applications found in /usr/share/applications
    pub programs: Vec<Rc<ProgramSuggestion>>,

    // files in $PATH
    pub executables: Vec<Rc<ExecutableSuggestion>>,
}

impl From<LanchCache> for LanchCacheRc {
    fn from(value: LanchCache) -> Self {
        Self {
            programs: value.programs.into_iter().map(Rc::new).collect(),
            executables: value.executables.into_iter().map(Rc::new).collect(),
        }
    }
}

impl LanchCache {
    fn generate_programs(
        icon_theme: Option<&str>,
    ) -> Result<Vec<ProgramSuggestion>, std::io::Error> {
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
                freedesktop_icons::lookup(fields[2]).with_size(48).find()
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
        println!(
            "[CACHE] generating new cache at {:?}",
            CACHE_FILE_PATH.clone()
        );

        let cache_dir = CACHE_FILE_PATH.parent().unwrap();
        if !cache_dir.exists() {
            println!("[CACHE] creating cache directory at {:?}", cache_dir);
            fs::create_dir_all(cache_dir)?;
        }

        let cache = Self {
            programs: Self::generate_programs(icon_theme)?,
            executables: Self::generate_executables()?,
        };

        let mut cache_file = File::create(CACHE_FILE_PATH.clone()).unwrap();

        let encoded: Vec<u8> = bincode::serialize(&cache).unwrap();
        cache_file.write_all(&encoded).unwrap();

        println!("[CACHE] done.");

        Ok(cache)
    }

    pub fn from_disk_or_new(icon_theme: Option<&str>) -> Result<Self, Box<dyn std::error::Error>> {
        let data = fs::read(CACHE_FILE_PATH.clone());
        match data {
            Ok(data) => {
                let decoded: LanchCache = bincode::deserialize(&data[..])?;
                Ok(decoded)
            }
            Err(_) => Self::new(icon_theme),
        }
    }
}
