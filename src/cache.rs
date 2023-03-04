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
    fn generate_programs() -> Result<Vec<ProgramSuggestion>, std::io::Error> {
        let mut ret: Vec<ProgramSuggestion> = Vec::new();

        for entry in fs::read_dir("/usr/share/applications")? {
            let file = fs::read_to_string(entry?.path())?;
            // TODO: this entire part is _very_ ugly,
            // there has to be a cleaner way
            let mut name: &str = "";
            let mut exec: &str = "";

            for line in file.lines() {
                if line.starts_with("Exec=") {
                    // There has to be an = sign, so this should never panic
                    let (_, exec_s) = line.split_once('=').unwrap();

                    if !exec_s.is_empty() {
                        exec = exec_s
                    }

                    if !name.is_empty() {
                        break;
                    }
                }

                if line.starts_with("Name=") {
                    // There has to be an = sign, so this should never panic
                    let (_, name_s) = line.split_once('=').unwrap();

                    if !name_s.is_empty() {
                        name = name_s;
                    }

                    if !exec.is_empty() {
                        break;
                    }
                }
            }

            if !name.is_empty() && !exec.is_empty() {
                ret.push(ProgramSuggestion::new(name, exec));
            }
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
    pub fn new() -> Result<Self, std::io::Error> {
        println!("[CACHE] generating new cache at {}", CACHE_DIR.clone());
        use std::fs::File;

        let cache = Self {
            programs: Self::generate_programs()?,
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

    pub fn from_disk_or_new() -> Result<Self, std::io::Error> {
        let data = fs::read(CACHE_DIR.clone());
        match data {
            Ok(data) => {
                let decoded: LanchCache = bincode::deserialize(&data[..]).unwrap();
                Ok(decoded)
            }
            Err(_) => {
                Self::new()
            }
        }
    }
}
