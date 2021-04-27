use std::env;
use std::fs;
use std::io::{Error, ErrorKind, Write};
use std::path::{Path, PathBuf};
use std::process::Command;

use chrono::{Datelike, Local, Timelike};
use clap::{crate_version, App, Arg};
use fs_extra::dir;

use serde::{Deserialize, Serialize};

const NOITA_SAVE_DIRECTORY: &str = "save00";

const NAUTTAJA_DIRECTORY: &str = ".nauttaja";
const NAUTTAJA_SAVES_DIRECTORY: &str = "saves";
const NAUTTAJA_LAST_REPLACED_DIRECTORY: &str = "backup";
const NAUTTAJA_TIMESTAMP_FILE: &str = "timestamp.txt";
const NAUTTAJA_CONFIG_FILE: &str = "config.json";

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Config {
    noita_root_dir: String,
}

fn main() {
    let mut app = App::new("nauttaja")
        .long_version(crate_version!())
        .subcommand(App::new("open").about("Open the game directory for Noita in explorer"))
        .subcommand(
            App::new("save")
                .about("Save the current game, with an optional custom name")
                .arg(Arg::new("name").about("Name of the new save")),
        )
        .subcommand(
            App::new("load")
                .about("Replace the current save with another saved game")
                .arg(Arg::new("name")),
        )
        .subcommand(App::new("list").about("Lists all currently saved games"))
        .subcommand(
            App::new("remove")
                .about("TODO: Remove a saved game")
                .arg(Arg::new("name").about("Name of the save to remove")),
        )
        .subcommand(
            App::new("set-noita-dir")
                .about("Set path to Noitas root directory")
                .arg(Arg::new("path").about("Path to Noitas root directory")),
        );

    let matches = app.clone().get_matches();

    if let Some(matches) = matches.subcommand_matches("set-noita-dir") {
        if let Some(path) = matches.value_of("path") {
            let config = Config {
                noita_root_dir: path.to_string(),
            };
            let nauttaja_dir = nauttaja_dir().expect("Failed to find home directory");
            let config_file = nauttaja_dir.join(NAUTTAJA_CONFIG_FILE);
            fs::create_dir_all(nauttaja_dir).expect("Failed to create savefile directory");
            fs::write(
                config_file,
                serde_json::to_string_pretty(&config).expect("Failed to serialize config"),
            )
            .expect("Unable to write file");
        } else {
            println!("Please specify the path to Noitas root directory");
        }
        return;
    }

    let config = load_config();
    if config.is_err() {
        println!(
            "Could not load config. Run nauttaja set-noita-dir <path to Noitas root directory>"
        );
        return;
    }
    let config = config.unwrap();

    if let Some(_) = matches.subcommand_matches("open") {
        open_explorer_in(&config.noita_root_dir);
    } else if let Some(matches) = matches.subcommand_matches("save") {
        if let Some(name) = matches.value_of("name") {
            save_game(&config, name);
        } else {
            println!("Please specify a name for the save");
        }
    } else if let Some(matches) = matches.subcommand_matches("load") {
        if let Some(name) = matches.value_of("name") {
            load_game(&config, name);
        } else {
            println!("Please specify which save to load");
            list_games().unwrap();
        }
    } else if let Some(_) = matches.subcommand_matches("list") {
        list_games().unwrap();
    } else if let Some(matches) = matches.subcommand_matches("remove") {
        println!(
            "TODO: Remove save with name: {:?}",
            matches.value_of("name")
        );
    } else {
        app.print_help().unwrap();
    }
}

fn save_game(config: &Config, save_name: &str) {
    println!("Saving game with name: {}", save_name);
    let work_dir = nauttaja_dir().expect("Failed to find home directory");
    let save_dir = work_dir.join(NAUTTAJA_SAVES_DIRECTORY).join(save_name);
    let time_file = save_dir.join(NAUTTAJA_TIMESTAMP_FILE);

    if save_dir.exists() {
        println!("[{}] already exists", save_name);
        return;
    }

    fs::create_dir_all(save_dir.clone()).expect("Failed to create savefile directory");

    let mut file = fs::File::create(time_file).expect("Could not create timestamp file");
    file.write_all(timestamp().as_bytes())
        .expect("Could not write timestamp");

    copy_dir(noita_save_dir(config), save_dir).expect("Failed to copy save");

    println!("Successfully saved game with name: {}", save_name)
}

fn load_game(config: &Config, save_name: &str) {
    println!("Loading game with name: {}", save_name);

    let work_dir = nauttaja_dir().expect("Failed to find home directory");
    let save_dir = work_dir.join(NAUTTAJA_SAVES_DIRECTORY).join(save_name);
    let backup_dir = work_dir.join(NAUTTAJA_LAST_REPLACED_DIRECTORY);

    if !save_dir.exists() {
        println!("Failed to find save with name: {}", save_name);
        return;
    }

    if backup_dir.exists() {
        fs::remove_dir_all(backup_dir.clone()).expect("Failed to remove last backup");
    }
    fs::create_dir(backup_dir.clone()).expect("Failed to create backup directory");

    copy_dir(noita_save_dir(config), backup_dir).expect("Failed to create emergency backup");

    fs::remove_dir_all(noita_save_dir(config)).expect("Failed to remove last save");

    copy_dir(
        save_dir.join(NOITA_SAVE_DIRECTORY),
        config.noita_root_dir.clone(),
    )
    .expect("Failed to load save");

    println!("Save [{}] successfully loaded!", save_name);
}

fn list_games() -> std::io::Result<()> {
    let work_dir = nauttaja_dir().expect("Failed to find home directory");
    let work_dir = work_dir.join(NAUTTAJA_SAVES_DIRECTORY);

    if !work_dir.exists() {
        println!("No saves found");
        return Ok(());
    }

    let mut saves = Vec::new();

    for entry in fs::read_dir(work_dir)? {
        let entry = entry?;
        let path = entry.path();
        let save_name = path
            .file_name()
            .and_then(|p| p.to_str())
            .map(String::from)
            .unwrap_or("INVALID_UTF8".to_string());
        if path.is_dir() {
            let timestamp = fs::read_to_string(path.join(NAUTTAJA_TIMESTAMP_FILE))?;
            saves.push((save_name, timestamp));
        }
    }

    saves.sort_by(|a, b| b.1.cmp(&a.1));

    saves.iter().for_each(|save| {
        println!("{} - {}", save.1, save.0);
    });

    Ok(())
}

fn copy_dir<A, B>(from: A, to: B) -> fs_extra::error::Result<u64>
where
    A: AsRef<Path>,
    B: AsRef<Path>,
{
    let options = dir::CopyOptions::new();
    let mut from_paths = Vec::new();
    from_paths.push(from);
    fs_extra::copy_items(&from_paths, to, &options)
}

fn nauttaja_dir() -> Option<PathBuf> {
    home::home_dir().map(|home_dir| home_dir.as_path().join(NAUTTAJA_DIRECTORY))
}

fn noita_save_dir(config: &Config) -> PathBuf {
    PathBuf::from(format!(
        "{}\\{}",
        config.noita_root_dir, NOITA_SAVE_DIRECTORY
    ))
}

fn load_config() -> Result<Config, ConfigError> {
    if let Some(dir) = nauttaja_dir() {
        let data = fs::read_to_string(dir.join(NAUTTAJA_CONFIG_FILE))?;
        let config = serde_json::from_str(&data)?;
        Ok(config)
    } else {
        Err(ConfigError::Io(Error::new(
            ErrorKind::NotFound,
            "Failed to find config file",
        )))
    }
}

fn open_explorer_in(dir: &str) {
    Command::new("explorer")
        .arg(dir)
        .spawn()
        .expect("Could not open explorer");
}

fn timestamp() -> String {
    let time = Local::now();
    format!(
        "{}-{:0>2}-{:0>2} {:0>2}:{:0>2}:{:0>2}",
        time.year(),
        time.month(),
        time.day(),
        time.time().hour(),
        time.time().minute(),
        time.time().second()
    )
}

#[derive(Debug)]
enum ConfigError {
    Io(std::io::Error),
    Serde(serde_json::Error),
}

impl From<std::io::Error> for ConfigError {
    fn from(io_error: std::io::Error) -> Self {
        ConfigError::Io(io_error)
    }
}

impl From<serde_json::Error> for ConfigError {
    fn from(serde_error: serde_json::Error) -> Self {
        ConfigError::Serde(serde_error)
    }
}
