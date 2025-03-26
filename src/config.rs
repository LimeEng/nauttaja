use serde::{Deserialize, Serialize};
use std::{
    env,
    path::{Path, PathBuf},
};

const NOITA_SAVE_DIRECTORY: &str = "save00";

const NAUTTAJA_DIRECTORY: &str = ".nauttaja";
const NAUTTAJA_CONFIG: &str = "config.json";
// const NAUTTAJA_SAVES_DIRECTORY: &str = "saves";
// const NAUTTAJA_LAST_REPLACED_DIRECTORY: &str = "backup";
// const NAUTTAJA_GAMEDB_FILE: &str = "gamedb.json";

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Config {
    pub noita_root_dir: PathBuf,
    pub open_folder_program: String,
    pub nauttaja_dir: PathBuf,
    pub nauttaja_config_file: PathBuf,
    pub noita_save_folder_name: PathBuf,
}

impl Config {
    #[must_use]
    fn new() -> Self {
        let home_dir = env::home_dir().expect("Failed to get home directory");
        let noita_root_dir = home_dir
            .join("Appdata")
            .join("LocalLow")
            .join("Nolla_Games_Noita");

        let nauttaja_dir = home_dir.join(".nauttaja");
        let nauttaja_config_file = home_dir.join("config.json");

        Self {
            noita_root_dir,
            open_folder_program: "explorer".to_string(),
            nauttaja_dir,
            nauttaja_config_file,
            noita_save_folder_name: Path::new("save00").to_path_buf(),
        }
    }

    // pub fn get() -> io::Result<Self> {
    //     let config_path = config_path()?;
    //     let content = fs::read_to_string(config_path)?;
    //     serde_json::from_str(&content).map_err(|_| io::Error::from(io::ErrorKind::InvalidData))
    // }

    // pub fn save(&self) -> io::Result<()> {
    //     let json =
    //         serde_json::to_string(self).map_err(|_| io::Error::from(io::ErrorKind::InvalidData))?;
    //     let config_path = config_path()?;
    //     let parent = config_path
    //         .parent()
    //         .ok_or_else(|| io::Error::from(io::ErrorKind::NotFound))?;
    //     fs::create_dir_all(parent)?;
    //     fs::write(config_path, json)
    // }

    #[must_use]
    pub fn active_save_path(&self) -> PathBuf {
        self.noita_root_dir.join(NOITA_SAVE_DIRECTORY)
    }
}

impl Default for Config {
    fn default() -> Self {
        Config::new()
    }
}
