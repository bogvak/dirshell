use std::collections::HashMap;
use std::env;
use std::fs;
use std::fs::File;
use std::fs::OpenOptions;
use std::io;
use std::io::prelude::*;
use std::io::BufReader;
use std::path::PathBuf;
use std::process::Child;
use std::process::Command;

use inquire::InquireError;
use inquire::Select;
use inquire::Text;

use reqwest;
use reqwest::Error;
use serde::Deserialize;

use chrono::prelude::*;

const EXCEPTION_COMMANDS: &'static [&'static str] = &["echo"];
static COM_HISTORY_FILENAME: &str = ".comhistory";

#[derive(Debug, Deserialize)]
struct Release {
    name: String,
    tag_name: String,
}

fn get_child_process(
    dirstring: &str,
    choice: &str,
    env_map: &HashMap<String, String>,
) -> io::Result<Child> {
    get_child_process_cmd(dirstring, choice, env_map, false)
}

fn get_child_process_cmd(
    dirstring: &str,
    choice: &str,
    env_map: &HashMap<String, String>,
    use_command: bool,
) -> io::Result<Child> {
    let mut parts: Vec<&str> = choice.split_whitespace().collect();
    let use_command_compound = use_command || EXCEPTION_COMMANDS.iter().any(|&s| s.eq(parts[0]));

    let command = match use_command_compound {
        true => {
            parts.insert(0, "/C");
            "cmd"
        }
        false => parts.remove(0),
    };

    // dbg!(&parts);
    let args = parts;

    match Command::new(command)
        .current_dir(dirstring)
        .args(args)
        .envs(env_map)
        .spawn()
    {
        Ok(child) => Ok(child),
        Err(e) => {
            println!("Failed to run command: {}", command);
            println!("Reason: {}", e);
            Err(e)
        }
    }
}

fn read_file(com_history_filename: &str) -> Result<Vec<String>, std::io::Error> {
    // let mut contents = vec![];
    let mut contents: Vec<String> = vec![];
    if std::path::Path::new(com_history_filename).exists() {
        let path = std::path::Path::new(com_history_filename);
        let file = std::fs::File::open(path)?;
        let command_lines: Vec<String> = std::io::BufReader::new(file)
            .lines()
            .map(|l| l.unwrap())
            .collect();
        // contents = command_lines.iter().map(|l| &l[..]).collect();
        contents = command_lines;
    }
    Ok(contents)
}

fn exit(exit_type: &str) {
    match exit_type {
        "ok" => std::process::exit(0),
        "err" => std::process::exit(1),
        _ => std::process::exit(0),
    }
}

fn search_for_env_files() -> Option<Vec<String>> {
    let mut result: Vec<String> = vec![];
    let dir = ".";
    let ext = "env";
    let entries = fs::read_dir(dir).unwrap();
    for entry in entries {
        let entry = entry.unwrap();
        let path = entry.path();
        if let Some(file_name) = path.file_name() {
            if let Some(ext_str) = file_name.to_str() {
                if ext_str.ends_with(ext) {
                    if let Some(name) = file_name.to_str() {
                        result.push(name.to_owned());
                    }
                }
            }
        }
    }
    if !result.is_empty() {
        Some(result)
    } else {
        None
    }
}

fn parse_key_value_pairs<'a>(lines: &'a Vec<String>) -> HashMap<&'a str, &'a str> {
    let mut map = HashMap::new();
    for line in lines {
        if let Some((key, value)) = line.split_once('=') {
            map.insert(key, value);
        }
    }
    map
}

fn read_env_file(file_path: &str) -> Vec<String> {
    let file = File::open(file_path).expect("Failed to open enviroment file");
    let reader = BufReader::new(file);
    reader
        .lines()
        .map(|line| line.expect("Failed to read data from enviroment"))
        .collect()
}

fn check_if_command_file_exists() -> bool {
    match fs::metadata(COM_HISTORY_FILENAME) {
        Ok(_) => true,
        Err(_) => false,
    }
}

fn get_current_directory() -> String {
    let dir = env::current_dir();
    match dir {
        Ok(path) => path.into_os_string().into_string().unwrap(),
        Err(_) => {
            exit("err");
            String::from("value")
        }
    }
}

fn get_env_variables_from_file() -> Option<HashMap<String, String>> {
    let env_files = search_for_env_files()?;
    let env_file_name = if env_files.len() == 1 {
        &env_files[0]
    } else {
        let contents: Vec<&str> = env_files.iter().map(|s| &**s).collect();
        let env_file_name_select: Result<&str, InquireError> =
            Select::new("envfile::", contents).prompt();
        match env_file_name_select {
            Ok(choice) => choice,
            Err(_) => return None,
        }
    };

    let env_lines = read_env_file(env_file_name);
    let vec_env_lines = env_lines.iter().map(|s| s.to_string()).collect();
    Some(
        parse_key_value_pairs(&vec_env_lines)
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect(),
    )
}

fn get_marker_modifed_date(marker: &PathBuf) -> String {
    let metadata = fs::metadata(marker).unwrap();
    let creation_time = metadata.modified().unwrap();
    let datetime: DateTime<Local> = DateTime::from(creation_time);
    return datetime.format("%Y-%m-%d").to_string();
}

fn if_need_to_check_new_version() -> bool {
    let mut marker = env::temp_dir();

    marker.push("dirshell.marker");
    if marker.exists() {
        let marker_date = get_marker_modifed_date(&marker);
        let today = Local::now();
        let today_date = today.format("%Y-%m-%d").to_string();
        if marker_date == today_date {
            return false;
        }
        fs::remove_file(&marker).expect("Can't create marker");
    }
    File::create(&marker).expect("Can't create marker");
    return true;
}

fn get_latest_available_release() -> Result<bool, Error> {
    if !if_need_to_check_new_version() {
        return Ok(true);
    };

    let current_version = env!("CARGO_PKG_VERSION");

    let owner = "bogvak";
    let repo = "dirshell";
    let url = format!(
        "https://api.github.com/repos/{}/{}/releases/latest",
        owner, repo
    );
    let client = reqwest::blocking::Client::new();
    let response = client
        .get(&url)
        .header("Accept", "application/vnd.github.v3+json")
        .header("User-Agent", "dirshell")
        .send()?;
    let latest_release_struct: Release = response.json()?;
    let release_name = &latest_release_struct.name[1..];

    if release_name != current_version {
        println!("\x1b[0;31mThere is a new version available: {}\x1b[0m", release_name);
        println!("You may download new version from project GitHub repository:");
        println!("\x1b[0;33mhttps://github.com/bogvak/dirshell/releases\x1b[0m")
    }
    Ok(true)
}
fn main() -> std::io::Result<()> {
    // Checking last available release
    let _ = get_latest_available_release();

    // Getting commandline arguments
    let mut args: Vec<String> = env::args().collect();
    args.drain(0..1);
    let mut mergedargs = args.join(" "); //.trim();

    let ignore_env = {
        if mergedargs == "--" {
            mergedargs = String::from("");
            true
        } else {
            false
        }
    };

    let needed_to_edit_command = {
        if mergedargs == "*" {
            mergedargs = String::from("");
            true
        } else {
            false
        }
    };

    // Getting current directory
    let dirstring = get_current_directory();
    println!("Current dir: {}", dirstring);

    if !check_if_command_file_exists() && mergedargs == "" {
        // if no history file and no args - just exit
        exit("ok");
    }

    // Read content of command file
    let contents_str = match read_file(COM_HISTORY_FILENAME) {
        Ok(res) => res,
        Err(_) => vec![],
    };
    let contents: Vec<&str> = contents_str.iter().map(|s| &**s).collect();
    let command_exists_in_file = contents.iter().any(|&s| s.eq(&mergedargs));

    if command_exists_in_file {
        exit("ok");
    };

    // updating history file and exit

    if mergedargs != "" {
        mergedargs.push('\n');
        let mut comfile = OpenOptions::new()
            .create(true)
            .append(true)
            .open(COM_HISTORY_FILENAME)?;
        comfile.write_all(mergedargs.as_bytes())?;
        exit("ok");
    }

    // Create copy for content for further saving and sort vector
    let mut contents_copy: Vec<String> = contents.iter().map(|&s| s.to_string()).collect();
    contents_copy.sort();

    // Show select dialog
    let mut ans = Select::new(">>", contents).prompt().unwrap_or_default();
    // if ans.is_empty() { exit("ok") };
    let inp = if needed_to_edit_command {
        let mut txt = Text::new("Command: ");
        txt.initial_value = Some(ans);
        txt.prompt().unwrap_or_default()
    } else {
        "".to_string()
    };
    ans = if needed_to_edit_command { &inp } else { ans };
    if ans.is_empty() {
        exit("ok")
    };

    // looking for .env files in current directory
    let env_hash = if !ignore_env {
        match get_env_variables_from_file() {
            Some(res) => res,
            None => HashMap::new(),
        }
    } else {
        HashMap::new()
    };

    // Remove selected, sort lines, insert selected as first element, write back to file
    if let Some(index) = contents_copy.iter().position(|el| *el == ans) {
        contents_copy.remove(index);
        contents_copy.insert(0, ans.to_string());
        let mut file = File::create(COM_HISTORY_FILENAME).expect("Failed to create file");
        for line in &contents_copy {
            file.write_all(line.as_bytes())
                .expect("Failed to write to file");
            file.write_all(b"\n").expect("Failed to write to file");
        }
    } else {
    }

    println!("{}", &ans);
    let cmd = match get_child_process(&dirstring, ans, &env_hash) {
        Ok(output) => output,
        Err(_e) => {
            println!("Retrying with shell: cmd /C {}", ans);
            match get_child_process_cmd(&dirstring, ans, &env_hash, true) {
                Ok(output) => output,
                Err(_e) => {
                    println!("Something wrong with that command");
                    panic!()
                }
            }
        }
    };

    let _output = match cmd.wait_with_output() {
        Ok(output) => output,
        Err(_e) => {
            println!("Failed to wait for command: {}", ans);
            panic!()
        }
    };

    Ok(())
}
