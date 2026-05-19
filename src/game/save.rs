use std::{fs::{self, File, OpenOptions}, path::PathBuf};

use directories::ProjectDirs;
use serde::{Deserialize, Serialize};


#[derive(Debug, Serialize, Deserialize)]
pub struct Save {
    last_finished_level: u8,
}

impl Default for Save {
    fn default() -> Self {
        Self { last_finished_level: 0 }
    }
}

impl Save {
    fn get_save_path() -> Option<PathBuf> {
        let dirs = ProjectDirs::from("com", "theNewSon", "Automation Game")?;
        let folder = dirs.data_dir();
    
        fs::create_dir_all(folder).ok()?;
    
        Some(folder.join("save.bin"))
    }

    pub fn load() -> Option<Self> {
        let path = Self::get_save_path()?;
        let file = File::open(path).ok()?;
        let save = bincode::deserialize_from(file).ok()?;
        Some(save)
    }

    pub fn save(&self) {
        let path = Self::get_save_path().unwrap();
        let file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(path)
            .unwrap();

        bincode::serialize_into(file, self).unwrap();
    }

    pub fn on_level_finish(&mut self, level: u8) {
        self.last_finished_level = self.last_finished_level.max(level);
    }

    pub fn get_last_finished_level(&self) -> u8 {
        self.last_finished_level
    }
}