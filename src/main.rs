use std::env;
use std::fs;
// use std::fs::File;
use std::fs::OpenOptions;
use std::io;
// use std::fs::OpenOptions;
use std::io::prelude::*;
use std::process::Child;
use std::process::Command;
// use std::process::Output;
// use std::str::SplitWhitespace;

use inquire::InquireError;
use inquire::Select;

// use std::path::Path;

const exception_commands: &'static [&'static str] = &["echo"];

fn get_child_process(dirstring: &str, choice: &str) -> io::Result<Child> {
    get_child_process_cmd(dirstring, choice, false)
}

fn get_child_process_cmd(dirstring: &str, choice: &str, use_command: bool) -> io::Result<Child> {
    
    let mut parts: Vec<&str> = choice.split_whitespace().collect();
    let use_command_compound = use_command
        || exception_commands
            .iter()
            .any(|&s| s.eq(parts[0]));

    let command = match use_command_compound {
        true => {parts.insert(0, "/C"); "cmd"},
        false => parts.remove(0),
    };

    // dbg!(&parts);
    let args = parts;

    match Command::new(command)
        .current_dir(dirstring)
        .args(args)
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

fn main() -> std::io::Result<()> {
    const COM_HISTORY_FILENAME: &str = ".comhistory";

    // Getting commandline arguments

    let mut args: Vec<String> = env::args().collect();
    args.drain(0..1);
    let mut mergedargs = args.join(" "); //.trim();

    // Getting current directory

    let dir = env::current_dir();
    let dirstring = match dir {
        Ok(path) => path.into_os_string().into_string().unwrap(),
        Err(_) => {
            exit("err");
            String::from("value")
        }
    };
    println!("Current dir: {}", dirstring);

    // Checking if history file exsts in current directory

    let file_exists = match fs::metadata(COM_HISTORY_FILENAME) {
        Ok(_) => true,
        Err(_) => false,
    };

    if !file_exists && mergedargs == "" {
        // if no history file and no args - just exit
        exit("ok");
    }

    let contents_str = match read_file(COM_HISTORY_FILENAME) {
        Ok(res) => res,
        Err(_) => vec![],
    };
    let contents: Vec<&str> = contents_str.iter().map(|s| &**s).collect();
    let line_exists = contents.iter().any(|&s| s.eq(&mergedargs));

    if line_exists {
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

    let ans: Result<&str, InquireError> = Select::new(">>", contents).prompt();

    match ans {
        Ok(choice) => {
            println!("{}", choice);
            let cmd = match get_child_process(&dirstring, choice) {
                Ok(output) => output,
                Err(_e) => {
                    println!("Retrying with shell: cmd /C {}", choice);
                    match get_child_process(&dirstring, choice) {
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
                    println!("Failed to wait for command: {}", choice);
                    panic!()
                }
            };
        }
        Err(_) => println!("There was an error, please try again"),
    };

    Ok(())
}
