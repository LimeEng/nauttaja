use clap::{Arg, ArgMatches, Command, crate_name, crate_version};
use nauttaja::storage::Storage;

fn main() {
    let mut app = Command::new(crate_name!())
        .version(crate_version!())
        .long_version(crate_version!())
        .arg_required_else_help(true)
        .about("Noita save management")
        .subcommand(
            Command::new("save").about("Save the current game").arg(
                Arg::new("name")
                    .help("Name of the new save")
                    .required(false),
            ),
        )
        .subcommand(
            Command::new("load")
                .about("Replace the current game with a saved game")
                .arg(
                    Arg::new("name")
                        .help("Name of the save to load")
                        .required(true),
                ),
        )
        .subcommand(Command::new("list").about("Lists all saved games"))
        .subcommand(
            Command::new("open")
                .about("Open the root directory in your file explorer")
                .arg(
                    Arg::new("noita")
                        .help("Open Noitas root directory in your file explorer")
                        .index(1)
                        .required(false)
                        .value_parser(["noita"]),
                ),
        );
    let matches = app.clone().get_matches();

    let storage = Storage::new();

    match matches.subcommand() {
        Some(("save", matches)) => save(&storage, matches.get_one::<String>("name").cloned()),
        Some(("load", matches)) => load(
            &storage,
            matches
                .get_one::<String>("name")
                .expect("Filename is mandatory"),
        ),
        Some(("list", _unused)) => list(storage),
        Some(("open", matches)) => open(&storage, matches),
        _ => app.print_help().unwrap(),
    };
}

fn save(storage: &Storage, name: Option<String>) {
    let _ = storage.save_as(name);
}

fn load(storage: &Storage, filename: &str) {
    println!("Creating emergency backup...");
    storage.backup_current_save().unwrap();
    println!("Backup created");
    println!("Loading");
    let _ = storage.load(filename);
}

fn list(storage: Storage) {
    // let t:Vec<_> = storage
    //     .saves()
    //     .iter()
    //     .flat_map(|saves| saves.iter().map(|save| save.metadata())).collect();

    let saves: Result<Vec<_>, _> = storage
        .saves()
        .and_then(|saves| saves.iter().map(|save| save.metadata()).collect());

    todo!()
    // match storage.saves() {
    //     Ok(saves) => {
    //         gamedb.saves.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));

    //         gamedb.saves.iter().for_each(|save| {
    //             println!("{} - {}", save.timestamp, save.name);
    //         });
    //     }
    //     Err(_) => println!("Failed to list saves"),
    // }
}

fn open(storage: &Storage, matches: &ArgMatches) {
    let status = match matches.get_one::<String>("noita").cloned() {
        Some(_) => storage.open_noita_folder(),
        None => storage.open_nauttaja_folder(),
    };
    // Even though the explorer window opens correctly,
    // the exit status is still 1 (not success)
    if (cfg!(target_os = "windows") && status.is_err())
        || (!cfg!(target_os = "windows") && !status.is_ok_and(|status| status.success()))
    {
        println!(
            "Failed to open the file explorer with [{}]",
            storage.config.open_folder_program
        );
        // TODO:
        // println!("You may try to set an alternative file explorer with");
        // println!("nauttaja config set [...]");
    }
}
