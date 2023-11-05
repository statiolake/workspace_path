use std::env;
use std::fs;
use std::path::{Path, PathBuf};

fn show_help() {
    let cmd_program = env::args()
        .next()
        .expect("failed to get the command which invoke this program (i.e. args[0])");

    println!("Utility for getting / creating workspace.");
    println!("Usage: {} [subcommand]", cmd_program);
    println!("Subcommands: ");
    println!("    temp (-t)  Get the directory template (copy source to create new workspace)");
    println!("    year (-y)  Get the year directory");
    println!("    date (-d)  Get today's workspace");
    println!("    help (-h)  Show this help message");
}

type Result<T> = std::result::Result<T, String>;
type UnitResult = Result<()>;

fn main() -> UnitResult {
    match env::args().nth(1).as_deref() {
        Some("temp") | Some("-t") => handle_temp(),
        Some("year") | Some("-y") => handle_year(),
        Some("date") | Some("-d") => handle_date(),
        Some("help") | Some("-h") => handle_help(),
        Some(subcmd) => handle_unknown(subcmd),
        None => handle_invalid(),
    }
}

fn handle_temp() -> UnitResult {
    println!("{}", workspace_path(WorkspacePathKind::Temp)?.display());
    Ok(())
}

fn handle_year() -> UnitResult {
    let year = workspace_path(WorkspacePathKind::Year)?;
    create_dirs(&year)?;
    println!("{}", year.display());
    Ok(())
}

fn handle_date() -> UnitResult {
    let temp = workspace_path(WorkspacePathKind::Temp)?;
    let year = workspace_path(WorkspacePathKind::Year)?;
    let date = workspace_path(WorkspacePathKind::Date)?;
    create_workspace_if_needed(&temp, &year, &date)?;
    println!("{}", date.display());
    Ok(())
}

fn handle_help() -> UnitResult {
    show_help();
    Ok(())
}

fn handle_unknown(subcmd: &str) -> UnitResult {
    show_help();
    Err(format!("unknown subcommand: {}", subcmd))
}

fn handle_invalid() -> UnitResult {
    show_help();
    Err("invalid number of arguments.".into())
}

enum WorkspacePathKind {
    Temp,
    Year,
    Date,
}

fn workspace_root() -> Result<PathBuf> {
    let mut root = dirs::home_dir().ok_or_else(|| "failed to get home dir".to_string())?;
    root.push("workspace");
    root.push("daily");

    // ジャンクションの場合があるので、その場合はジャンクションを解決したパスを返す。
    to_absolute::canonicalize(root).map_err(|e| format!("canonicalization failed: {:?}", e))
}

fn workspace_path(kind: WorkspacePathKind) -> Result<PathBuf> {
    use chrono::Local;

    let mut result = workspace_root()?;

    let now = Local::now();
    let year = now.format("%Y").to_string();
    let date = now.format("%m%d").to_string();

    match kind {
        WorkspacePathKind::Temp => result.push("template"),
        WorkspacePathKind::Year => result.push(year),
        WorkspacePathKind::Date => result.extend(&[year, date]),
    }

    Ok(result)
}

fn create_workspace_if_needed(temp: &Path, year: &Path, date: &Path) -> UnitResult {
    if date.exists() {
        return Ok(());
    }

    if !temp.exists() {
        return Err("workspace template directory does not exist.".into());
    }

    create_dirs(year)?;
    use fs_extra::dir::{copy, CopyOptions};
    copy(temp, year, &CopyOptions::new())
        .map_err(|e| format!("failed to copy template directory: {}", e))?;
    let copied = year.join("template");
    fs::rename(copied, date).map_err(|e| format!("failed to rename copied directory: {}", e))?;

    Ok(())
}

fn create_dirs(dir: &Path) -> UnitResult {
    fs::create_dir_all(dir).map_err(|x| format!("failed to create directory: {}", x))
}
