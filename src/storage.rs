use crate::{
    archive::{self, Metadata},
    config::Config,
};
use std::{
    ffi::OsStr,
    fs::{self, File},
    io,
    path::PathBuf,
    process::{self, Stdio},
};

const NAUTTAJA_DIRECTORY: &str = ".nauttaja2";
const NAUTTAJA_CONFIG: &str = "config.json";
const NAUTTAJA_SAVES_DIRECTORY: &str = "saves";
const NAUTTAJA_LAST_REPLACED_DIRECTORY: &str = "backup";

pub struct Storage {
    pub config: Config,
}

impl Storage {
    #[must_use]
    pub fn new() -> Self {
        let config = config_path()
            .and_then(|path| fs::read_to_string(path).ok())
            .and_then(|json| serde_json::from_str(&json).ok())
            .unwrap_or_default();

        Self { config }
    }

    pub fn save_as(&self, filename: &str, metadata: Metadata) -> io::Result<()> {
        let nauttaja_dir = nauttaja_directory().unwrap();
        let file = File::create_new(nauttaja_dir.join(filename))?;
        archive::bundle(self.config.active_save_path(), file, &metadata)
    }

    pub fn load(&self, filename: &str) -> io::Result<()> {
        let nauttaja_dir = nauttaja_directory().unwrap();
        let save = nauttaja_dir.join(filename);
        let save = File::open(save)?;
        archive::unpack(save, self.config.noita_root_dir.clone())
    }

    pub fn save_config(&self) -> io::Result<()> {
        let json = serde_json::to_string(&self.config)
            .map_err(|_| io::Error::from(io::ErrorKind::InvalidData))?;

        let config_path = config_path().ok_or_else(|| io::Error::from(io::ErrorKind::NotFound))?;
        let parent = config_path
            .parent()
            .ok_or_else(|| io::Error::from(io::ErrorKind::NotFound))?;
        fs::create_dir_all(parent)?;
        fs::write(config_path, json)
    }

    // pub fn saves() -> Vec<PathBuf> {
    //     let archive_path = home::home_dir()
    //         .map(|home_dir| {
    //             home_dir
    //                 .as_path()
    //                 .join(NAUTTAJA_DIRECTORY)
    //                 .join(NAUTTAJA_SAVES_DIRECTORY)
    //         })
    //         .unwrap();
    //     let paths = fs::read_dir(archive_path).unwrap();

    //     paths
    //         .into_iter()
    //         .filter_map(Result::ok)
    //         .filter(|entry| {
    //             if let Ok(metadata) = entry.metadata() {
    //                 metadata.is_file()
    //                     && entry.path().extension().is_some_and(|ext| ext == "tar.gz")
    //             } else {
    //                 false
    //             }
    //         })
    //         .map(|entry| entry.path())
    //         .collect()
    // }
    pub fn open_nauttaja_folder(&self) -> Result<process::ExitStatus, io::Error> {
        let dir = nauttaja_directory().ok_or_else(|| io::Error::from(io::ErrorKind::NotFound))?;
        self.open_folder(dir)
    }

    pub fn open_noita_folder(&self) -> Result<process::ExitStatus, io::Error> {
        self.open_folder(self.config.noita_root_dir.clone())
    }

    fn open_folder<S: AsRef<OsStr>>(&self, directory: S) -> Result<process::ExitStatus, io::Error> {
        process::Command::new(self.config.open_folder_program.clone())
            .arg(directory)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
    }
}

fn nauttaja_directory() -> Option<PathBuf> {
    home::home_dir().map(|home_dir| home_dir.as_path().join(NAUTTAJA_DIRECTORY))
}

fn config_path() -> Option<PathBuf> {
    nauttaja_directory().map(|path| path.join(NAUTTAJA_CONFIG))
}
