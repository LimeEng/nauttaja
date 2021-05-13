use std::env;
use std::fs;
use std::io::{Error, ErrorKind};
use std::path::{Path, PathBuf};
use std::process::Command;

use chrono::{Datelike, Local, Timelike};
use clap::{crate_version, App, Arg};
use fs_extra::dir;
use uuid::Uuid;

use serde::{Deserialize, Serialize};

const NOITA_SAVE_DIRECTORY: &str = "save00";

const NAUTTAJA_DIRECTORY: &str = ".nauttaja";
const NAUTTAJA_SAVES_DIRECTORY: &str = "saves";
const NAUTTAJA_LAST_REPLACED_DIRECTORY: &str = "backup";
const NAUTTAJA_GAMEDB_FILE: &str = "gamedb.json";

const CLI_SUBCMD_OPEN_OPTIONS: [&str; 2] = ["noita", "nauttaja"];

#[derive(Serialize, Deserialize, Default, Debug, Clone)]
struct Config {
    noita_root_dir: String,
}

#[derive(Serialize, Deserialize, Default, Debug, Clone)]
struct GameDB {
    saves: Vec<Save>,
    trash: Vec<Save>,
    config: Config,
}

#[derive(Serialize, Deserialize, Default, Debug, Clone)]
struct Save {
    name: String,
    directory: String,
    timestamp: String,
}

fn main() {
    let mut app = App::new("nauttaja")
        .version(crate_version!())
        .long_version(crate_version!())
        .subcommand(
            App::new("open")
                .about("Open the specified target in explorer")
                .arg(
                    Arg::new("target")
                        .possible_values(&CLI_SUBCMD_OPEN_OPTIONS)
                        .required(true),
                ),
        )
        .subcommand(
            App::new("save")
                .about("Save the current game, with an optional custom name")
                .arg(
                    Arg::new("name")
                        .about("Name of the new save")
                        .required(true),
                ),
        )
        .subcommand(
            App::new("load")
                .about("Replace the current save with another saved game")
                .arg(Arg::new("name").about("Name of the save to load")),
        )
        .subcommand(
            App::new("list")
                .about("Lists all currently saved games")
                .arg(
                    Arg::new("removed")
                        .about("Lists all removed saves")
                        .index(1)
                        .possible_value("removed")
                        .required(false),
                ),
        )
        .subcommand(
            App::new("remove")
                .about("Remove the specified save")
                .arg(Arg::new("name").about("Name of the save to remove")),
        )
        .subcommand(
            App::new("restore")
                .about("Restores the specified save")
                .arg(Arg::new("name").about("Name of the save to restore")),
        )
        .subcommand(
            App::new("delete")
                .about("Permanently deletes the specified save")
                .arg(
                    Arg::new("name")
                        .about("Name of the save to permanently delete"),
                ),
        )
        .subcommand(
            App::new("import")
                .about("Imports a directory as a save")
                .arg(
                    Arg::new("name")
                        .about("Name of the new save")
                        .required(true),
                )
                .arg(
                    Arg::new("path")
                        .about("Path to the directory to import")
                        .required(true),
                ),
        )
        .subcommand(
            App::new("set-noita-dir")
                .about("Set path to Noitas root directory")
                .arg(
                    Arg::new("path")
                        .about("Path to Noitas root directory")
                        .required(true),
                ),
        );

    let matches = app.clone().get_matches();

    if let Some(matches) = matches.subcommand_matches("set-noita-dir") {
        let path = matches.value_of("path").unwrap(); // Required argument
        update_noita_dir(path);
        return;
    }

    let gamedb = load_gamedb();
    if gamedb.is_err() {
        println!(
            "Could not load gamedb. Run nauttaja set-noita-dir <path to Noitas root directory>"
        );
        return;
    }
    let gamedb = gamedb.unwrap();

    if let Some(matches) = matches.subcommand_matches("open") {
        match matches.value_of("target").unwrap() {
            // Required argument
            "noita" => open_explorer_in(&gamedb.config.noita_root_dir),
            "nauttaja" => open_explorer_in(
                nauttaja_dir()
                    .expect("Failed to find home directory")
                    .to_str()
                    .expect("Path contains invalid UTF8"),
            ),
            _ => panic!("Unrecognized target"),
        };
    } else if let Some(matches) = matches.subcommand_matches("save") {
        let name = matches.value_of("name").unwrap(); // Required argument
        save_game(&gamedb.config, name).expect("Failed to save game");
    } else if let Some(matches) = matches.subcommand_matches("load") {
        if let Some(name) = matches.value_of("name") {
            load_save(&gamedb.config, name).expect("Failed to load save");
        } else {
            println!("Please specify which save to load");
            list_saves().expect("Failed to list saves");
        }
    } else if let Some(matches) = matches.subcommand_matches("list") {
        if matches.is_present("removed") {
            list_trash()
        } else {
            list_saves()
        }
        .expect("Failed to list saves");
    } else if let Some(matches) = matches.subcommand_matches("remove") {
        if let Some(name) = matches.value_of("name") {
            remove_save(name).expect("Failed to remove save");
        } else {
            println!("Please specify which save to remove");
            list_trash().expect("Failed to list saves");
        }
    } else if let Some(matches) = matches.subcommand_matches("restore") {
        if let Some(name) = matches.value_of("name") {
            restore_save(name).expect("Failed to restore save");
        } else {
            println!("Please specify which save to restore");
            list_trash().expect("Failed to list saves");
        }
    } else if let Some(matches) = matches.subcommand_matches("delete") {
        if let Some(name) = matches.value_of("name") {
            delete_save(name).expect("Failed to delete save");
        } else {
            println!("Please specify which save to permanently delete");
            println!("Note that you can only permanently delete removed saves");
            list_trash().expect("Failed to list saves");
        }
    } else if let Some(matches) = matches.subcommand_matches("import") {
        let path = matches.value_of("path").unwrap(); // Required argument
        let name = matches.value_of("name").unwrap(); // Required argument
        import_save(path, name).expect("Failed to import save");
    } else {
        app.print_help().unwrap();
    }
}

fn update_noita_dir(noita_path: &str) {
    update_gamedb(|mut gamedb: GameDB| {
        gamedb.config.noita_root_dir = noita_path.to_string();
        gamedb
    })
    .expect("Failed to update Noita directory");
}

fn update_gamedb<T>(mut update_fn: T) -> Result<(), CliError>
where
    T: FnMut(GameDB) -> GameDB,
{
    let nauttaja_dir = nauttaja_dir()?;
    let gamedb_file = nauttaja_dir.join(NAUTTAJA_GAMEDB_FILE);
    fs::create_dir_all(nauttaja_dir)?;
    let gamedb = if gamedb_file.exists() {
        load_gamedb()?
    } else {
        GameDB {
            ..Default::default()
        }
    };

    let gamedb = update_fn(gamedb);
    fs::write(gamedb_file, serde_json::to_string_pretty(&gamedb)?)?;
    Ok(())
}

fn delete_save(save_name: &str) -> Result<(), CliError> {
    println!("Deleting save with name [{}]", save_name);

    let mut dir_to_delete = None;
    update_gamedb(|mut gamedb: GameDB| {
        let index = gamedb.trash.iter().position(|item| item.name == save_name);
        if index.is_none() {
            let index = gamedb.saves.iter().position(|item| item.name == save_name);
            match index {
                Some(_) => {
                    println!("Found save [{}], currently not in the trash", save_name);
                    println!("To permanently delete this save, please trash it first");
                }
                None => println!("Failed to find [{}]", save_name),
            }
        } else {
            let deleted = gamedb.trash.remove(index.unwrap());
            dir_to_delete = Some(deleted.directory);
        }
        gamedb
    })?;

    if dir_to_delete.is_some() {
        let dir = dir_to_delete.unwrap();
        let work_dir = nauttaja_dir()?;
        let save_dir = work_dir.join(NAUTTAJA_SAVES_DIRECTORY).join(dir);

        fs::remove_dir_all(save_dir)?;
        println!("Deleted save successfully");
    }

    Ok(())
}

fn remove_save(save_name: &str) -> Result<(), CliError> {
    println!("Removing save with name [{}]", save_name);
    update_gamedb(|mut gamedb: GameDB| {
        let index = gamedb.saves.iter().position(|item| item.name == save_name);
        if index.is_none() {
            println!("Failed to find [{}]", save_name);
        } else {
            let index = index.unwrap();
            gamedb.trash.push(gamedb.saves.remove(index));
        }
        gamedb
    })?;
    println!("Save with name [{}] removed", save_name);
    Ok(())
}

fn restore_save(save_name: &str) -> Result<(), CliError> {
    println!("Restoring save with name [{}]", save_name);
    update_gamedb(|mut gamedb: GameDB| {
        let index = gamedb.trash.iter().position(|item| item.name == save_name);
        if index.is_none() {
            println!("Failed to find [{}]", save_name);
        } else {
            let index = index.unwrap();
            gamedb.saves.push(gamedb.trash.remove(index));
        }
        gamedb
    })?;
    println!("Save with name [{}] restored", save_name);
    Ok(())
}

fn import_save(directory: &str, save_name: &str) -> Result<(), CliError> {
    println!(
        "Importing directory [{}] as a new save, named [{}]",
        directory, save_name
    );
    save_dir_as_save(directory, save_name)?;
    println!(
        "Successfully imported directory as a save with name [{}]",
        directory
    );
    Ok(())
}

fn save_game(config: &Config, save_name: &str) -> Result<(), CliError> {
    println!("Saving game with name [{}]", save_name);
    save_dir_as_save(noita_save_dir(config), save_name)?;
    println!("Successfully saved game with name [{}]", save_name);
    Ok(())
}

fn save_dir_as_save<T>(directory: T, save_name: &str) -> Result<(), CliError>
where
    T: AsRef<Path>,
{
    let gamedb = load_gamedb()?;

    if gamedb.saves.iter().any(|item| item.name == save_name) {
        println!("[{}] already exists", save_name);
        return Ok(());
    }
    if gamedb.trash.iter().any(|item| item.name == save_name) {
        println!("[{}] already exists, currently in the trash", save_name);
        return Ok(());
    }

    let work_dir = nauttaja_dir()?;
    let save_dir_name = uuid();
    let save_dir = work_dir
        .join(NAUTTAJA_SAVES_DIRECTORY)
        .join(save_dir_name.clone());

    fs::create_dir_all(save_dir.clone())?;

    copy_dir(directory, save_dir)?;

    update_gamedb(|mut gamedb: GameDB| {
        let save = Save {
            name: save_name.to_string(),
            directory: save_dir_name.clone(),
            timestamp: timestamp(),
        };
        gamedb.saves.push(save);
        gamedb
    })?;

    Ok(())
}

fn load_save(config: &Config, save_name: &str) -> Result<(), CliError> {
    println!("Loading save with name [{}]", save_name);

    let work_dir = nauttaja_dir()?;
    let backup_dir = work_dir.join(NAUTTAJA_LAST_REPLACED_DIRECTORY);
    let gamedb = load_gamedb()?;
    let save = gamedb.saves.iter().find(|item| item.name == save_name);
    if save.is_none() {
        println!("Failed to find save with name [{}]", save_name);
        return Ok(());
    }
    let save = save.unwrap();
    let save_dir = work_dir
        .join(NAUTTAJA_SAVES_DIRECTORY)
        .join(save.directory.clone());

    if !save_dir.exists() {
        println!("Failed to find save with name [{}]", save_name);
        return Ok(());
    }

    if backup_dir.exists() {
        fs::remove_dir_all(backup_dir.clone())?;
    }
    fs::create_dir(backup_dir.clone())?;

    println!("Creating emergency backup...");
    copy_dir(noita_save_dir(config), backup_dir)?;

    println!("Loading [{}]...", save_name);
    fs::remove_dir_all(noita_save_dir(config))?;

    copy_dir(
        save_dir.join(NOITA_SAVE_DIRECTORY),
        config.noita_root_dir.clone(),
    )?;

    println!("Save [{}] successfully loaded!", save_name);
    Ok(())
}

fn list_saves() -> Result<(), CliError> {
    let mut gamedb = load_gamedb()?;

    if gamedb.saves.is_empty() {
        println!("No saves found");
        return Ok(());
    }

    gamedb.saves.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));

    gamedb.saves.iter().for_each(|save| {
        println!("{} - {}", save.timestamp, save.name);
    });

    Ok(())
}

fn list_trash() -> Result<(), CliError> {
    let mut gamedb = load_gamedb()?;

    if gamedb.trash.is_empty() {
        println!("No saves found");
        return Ok(());
    }

    gamedb.trash.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));

    gamedb.trash.iter().for_each(|save| {
        println!("{} - {}", save.timestamp, save.name);
    });

    Ok(())
}

fn copy_dir<A, B>(from: A, to: B) -> Result<(), CliError>
where
    A: AsRef<Path>,
    B: AsRef<Path>,
{
    let options = dir::CopyOptions::new();
    let mut from_paths = Vec::new();
    from_paths.push(from);
    fs_extra::copy_items(&from_paths, to, &options)?;
    Ok(())
}

fn nauttaja_dir() -> std::io::Result<PathBuf> {
    home::home_dir()
        .ok_or(Error::new(
            ErrorKind::NotFound,
            "Failed to find the home directory",
        ))
        .map(|home_dir| home_dir.as_path().join(NAUTTAJA_DIRECTORY))
}

fn noita_save_dir(config: &Config) -> PathBuf {
    PathBuf::from(format!(
        "{}\\{}",
        config.noita_root_dir, NOITA_SAVE_DIRECTORY
    ))
}

fn load_gamedb() -> Result<GameDB, CliError> {
    if let Ok(dir) = nauttaja_dir() {
        let data = fs::read_to_string(dir.join(NAUTTAJA_GAMEDB_FILE))?;
        let config = serde_json::from_str(&data)?;
        Ok(config)
    } else {
        Err(CliError::Io(Error::new(
            ErrorKind::NotFound,
            "Failed to find gamedb file",
        )))
    }
}

fn uuid() -> String {
    Uuid::new_v4().to_hyphenated().to_string()
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
enum CliError {
    Io(std::io::Error),
    Serde(serde_json::Error),
    FsExtra(fs_extra::error::Error),
}

impl From<std::io::Error> for CliError {
    fn from(error: std::io::Error) -> Self {
        CliError::Io(error)
    }
}

impl From<serde_json::Error> for CliError {
    fn from(error: serde_json::Error) -> Self {
        CliError::Serde(error)
    }
}

impl From<fs_extra::error::Error> for CliError {
    fn from(error: fs_extra::error::Error) -> Self {
        CliError::FsExtra(error)
    }
}
