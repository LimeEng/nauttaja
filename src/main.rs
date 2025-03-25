use clap::{crate_name, crate_version, Arg, ArgMatches, Command};
use nauttaja::{archive::Metadata, storage::Storage};
use time::{macros::format_description, OffsetDateTime};

fn main() {
    let mut app = Command::new(crate_name!())
        .version(crate_version!())
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
        Some(("save", matches)) => save(storage, matches.get_one::<String>("name").cloned()),
        Some(("load", matches)) => load(
            storage,
            matches
                .get_one::<String>("name")
                .expect("Filename is mandatory"),
        ),
        Some(("list", _unused)) => list(storage),
        Some(("open", matches)) => open(storage, matches),
        _ => app.print_help().unwrap(),
    };
}

fn save(storage: Storage, name: Option<String>) {
    let format = format_description!("[year]-[month]-[day] [hour].[minute].[second]");
    let timestamp = OffsetDateTime::now_utc();
    let metadata = Metadata {
        name,
        notes: None,
        timestamp,
    };

    let _ = storage.save_as(
        &timestamp.format(format).expect("Failed to format time"),
        metadata,
    );
}

fn load(storage: Storage, filename: &str) {
    let _ = storage.load(filename);
}

fn list(_storage: Storage) {}

fn open(storage: Storage, matches: &ArgMatches) {
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
