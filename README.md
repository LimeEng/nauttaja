![CI status](https://github.com/LimeEng/nauttaja/workflows/CI/badge.svg)

# Nauttaja

Derived from the [allegedly Finnish word for helper](https://translate.google.se/?sl=fi&tl=en&text=auttaja&op=translate), nauttaja is a save management tool for [Noita](https://store.steampowered.com/app/881100/Noita/). The primary purpose for this tool is to circumvent Noitas perma-death by backing up the game files and restoring them whenever needed.

Please do note that this is a hacked together mess and since it features automatic file deletion it can be potentially dangerous.

## Table of Contents
- [Usage](#usage)
- [Installation](#installation)

## Usage

After [installation](#installation), the tool requires a one-time setup. You need to supply it with Noitas root directory on your system. On Windows, this is typically located at `C:\Users\<username>\AppData\LocalLow\Nolla_Games_Noita`. Once you have the path, simply run the following command:
```
nauttaja set-noita-dir <path to Noitas root directory>
```
That's it! The tool is now fully configured.

### **Important!**

To ensure smooth operation, only run this tool when you are sure that no other programs are accessing Noitas files. Do not run the tool while Noita is running or Steam Cloud Sync is trying to save your progress.

### Commands

- **`nauttaja`**

    Running the tool without any arguments will print a helpful summary of all available commands.

- **`nauttaja save <name>`**

    This will create a new save with the specified name.

- **`nauttaja load <name>`**

    This will load the specified save by replacing whatever save is currently loaded. Since this is a potentially destructive operation the tool will first try and create a backup, located at `~/.nauttaja/backup`. The backup is deleted and replaced whenever `nauttaja load` is run again. Currently, this backup must be manually restored if necessary.

- **`nauttaja remove <name>`**

    This will remove the specified save, placing it in the "trash". It can still be restored but cannot be used until then.

- **`nauttaja restore <name>`**

    This will restore the specified save, removing it from the "trash" and placing it among the other saves.

- **`nauttaja delete <name>`**

    This will **permanently** delete the specified save. You can only delete saves which currently are in the "trash", placed there by the `remove`-command.

- **`nauttaja list`**

    This will list all available saves, sorted by time created.

- **`nauttaja list removed`**

    This will list all removed saves, sorted by time created.

- **`nauttaja open`**

    This will open nauttajas root directory in Windows explorer. Since this is dependent on Windows-specific functionality, this command will not work on other platforms.

- **`nauttaja open noita`**

    This will open Noitas root directory in Windows explorer. Since this is dependent on Windows-specific functionality, this command will not work on other platforms.

- **`nauttaja import <name> <path>`**

    It can be useful to import external directories as if they were saved from Noitas game directory. This command imports the specified directory and saves it as a normal save with the specified name.

## Installation

To use the tool, you can download a pre-built binary from the [releases-page](https://github.com/LimeEng/nauttaja/releases). Do note that Noita is only officially supported on Windows.

Another possibility, if [cargo and rust are installed](https://www.rust-lang.org/tools/install), is to download and install the latest commit on master by running the following command:
```
cargo install --git https://github.com/LimeEng/nauttaja
```
