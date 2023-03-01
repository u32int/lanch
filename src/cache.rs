use super::*;
use serde::{Deserialize, Serialize};
use std::io::prelude::*;

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct LanchCache {
    // Programs are applications found in /usr/share/applications
    pub programs: Vec<ProgramSuggestion>,
}

impl LanchCache {
    const CACHE_DIR: &str = "/home/vsh/.cache/lanch/cachefile";

    fn generate_programs() -> Result<Vec<ProgramSuggestion>, std::io::Error> {
        use std::fs;
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
                    let (_, exec_s) = line.split_once("=").unwrap();

                    if !exec_s.is_empty() {
                        exec = exec_s
                    }

                    if !name.is_empty() {
                        break
                    }
                }

                if line.starts_with("Name=") {
                    // There has to be an = sign, so this should never panic
                    let (_, name_s) = line.split_once("=").unwrap();

                    if !name_s.is_empty() {
                        name = name_s; 
                    }

                    if !exec.is_empty() {
                        break
                    }
                }
            }

            if !name.is_empty() && !exec.is_empty() {
                ret.push(ProgramSuggestion::new(&name[0..10], &exec[0..10]));
            }
        }

        Ok(ret)
    }

    // Generates new cache and writes it to disk
    pub fn new() -> Result<Self, std::io::Error> {
        use std::fs::File;

        // TODO: gen actual cache
        let cache = Self {
            programs: Self::generate_programs()?,
        };

        let mut cache_file = match File::open(LanchCache::CACHE_DIR) {
            Ok(f) => f,
            Err(_) => {

                File::create(LanchCache::CACHE_DIR).unwrap()
            }
        };

        let encoded: Vec<u8> = bincode::serialize(&cache).unwrap();
        cache_file.write_all(&encoded).unwrap();

        Ok(cache)
    }

    pub fn from_disk_or_new() -> Result<Self, std::io::Error> {
        use std::fs;

        let data = fs::read(LanchCache::CACHE_DIR);
        match data {
            Ok(data) => {
                let decoded: LanchCache = bincode::deserialize(&data[..]).unwrap();
                return Ok(decoded);
            }
            Err(_) => {
                return Self::new();
            }
        }
    }
}
