use std::{fs, path::PathBuf};

use clap::{Parser, Subcommand};
use steam_shortcuts_util::{parse_shortcuts, shortcuts_to_bytes, Shortcut};
use steamlocate::SteamDir;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// List all non-Steam games
    List,

    /// Add a non-Steam game to the library
    Add {
        // Name of the game
        name: String,

        // Command to run the game
        executable: String,
    },

    /// Remove a non-Steam game from the library
    Remove {
        // Name of the game
        name: String,
    },
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Command::List => list_games(),
        Command::Add { name, executable } => add_game(name, executable),
        Command::Remove { name } => remove_game(name),
    }
}

fn get_non_steam_games() -> Vec<steamlocate::Shortcut> {
    if let Some(mut steam_dir) = SteamDir::locate() {
        return steam_dir.shortcuts().to_vec();
    }

    vec![]
}

fn list_games() {
    let games = get_non_steam_games();
    let max_width = games.iter()
        .map(|s| s.app_name.len())
        .max()
        .unwrap_or(0)
        .max(4);

    let padding = max_width + 4;
    eprintln!("{:padding$}{}", "GAME", "EXECUTABLE", padding = padding);

    for game in games {
        println!("{:padding$}{}", game.app_name, game.executable, padding = padding);
    }
}

fn get_shortcuts_file() -> Option<PathBuf> {
    let steam_dir = SteamDir::locate().unwrap();
    let user_data = steam_dir.path.join("userdata");

    for entry in fs::read_dir(user_data).ok()?.filter_map(|e| e.ok()) {
        let shortcuts_path = entry.path().join("config").join("shortcuts.vdf");
        if shortcuts_path.is_file() {
            return Some(shortcuts_path);
        }
    }

    None
}

fn add_game(name: String, executable: String) {
    let shortcuts_file = get_shortcuts_file().unwrap();
    let content = std::fs::read(shortcuts_file.clone()).unwrap();

    let mut shortcuts = parse_shortcuts(&content).unwrap();

    if shortcuts.iter().any(|s| s.app_name == name) {
        eprintln!("Game already exists");
        return;
    }


    let order = shortcuts.len().to_string();
    shortcuts.push(Shortcut::new(
        &order,
        &name,
        &executable,
        "\"./\"",
        "",
        "",
        ""
    ));

    let bytes = shortcuts_to_bytes(&shortcuts);
    std::fs::write(shortcuts_file, bytes).unwrap();
}

fn remove_game<'a>(name: String) {
    let shortcuts_file = get_shortcuts_file().unwrap();
    let content = std::fs::read(shortcuts_file.clone()).unwrap();

    let shortcuts = parse_shortcuts(&content).unwrap();

    if !shortcuts.iter().any(|s| s.app_name == name) {
        eprintln!("Game does not exist");
        return;
    }

    let shortcuts: Vec<_> = shortcuts.into_iter()
        .filter(|s| s.app_name != name)
        .map(|s| s.to_owned())
        .enumerate()
        .map(|(i, mut s)| {
            s.order = i.to_string();
            s
        })
        .collect::<Vec<_>>();

    let bytes = shortcuts_to_bytes(&shortcuts.iter().map(|s| s.borrow()).collect());
    std::fs::write(shortcuts_file, bytes).unwrap();
}
