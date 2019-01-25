mod rusty_hook;

use std::fs::File;
use std::io::prelude::*;
#[cfg(target_family = "unix")]
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::{exit, Command};

extern crate ci_info;
// extern crate getopts;
// use getopts::Options;
use std::env;

fn get_command_runner() -> fn(cmd: &str) -> Result<String, String> {
    |cmd: &str| {
        let mut program = "sh";
        let mut switch = "-c";
        if cfg!(target_os = "windows") {
            program = "cmd";
            switch = "/C";
        };
        match Command::new(&program).args(&[&switch, cmd]).output() {
            Err(details) => panic!(
                "Command runner crashed in unrecoverable manner. Details: {}",
                details
            ),
            Ok(output) => {
                if output.status.success() {
                    Ok(String::from_utf8(output.stdout).unwrap().replace("\n", ""))
                } else {
                    Err(String::from_utf8(output.stderr).unwrap())
                }
            }
        }
    }
}

#[cfg(target_family = "unix")]
fn create_file(path: PathBuf, make_executable: bool) -> Result<File, ()> {
    let file = match File::create(&path) {
        Ok(file) => file,
        Err(_) => return Err(()),
    };

    if make_executable {
        let metadata = match file.metadata() {
            Ok(metadata) => metadata,
            Err(_) => return Err(()),
        };

        let mut permissions = metadata.permissions();
        permissions.set_mode(0o755);
    };

    Ok(file)
}

#[cfg(target_family = "windows")]
fn create_file(path: PathBuf, _make_executable: bool) -> Result<File, ()> {
    match File::create(&path) {
        Err(_) => Err(()),
        Ok(file) => Ok(file),
    }
}

fn get_file_writer(
) -> fn(file_path: &str, contents: &str, make_executable: bool) -> Result<(), String> {
    |file_path: &str, contents: &str, make_executable: bool| {
        let path = PathBuf::from(file_path);
        let mut file = match create_file(path, make_executable) {
            Ok(f) => f,
            Err(_) => return Err(format!("Failed to create file {}", file_path)),
        };

        match file.write_all(contents.as_bytes()) {
            Ok(_) => Ok(()),
            Err(_) => Err(format!("Failed to write contents to {}", file_path)),
        }
    }
}

fn get_file_existence_checker() -> fn(file_path: &str) -> Result<bool, ()> {
    |file_path: &str| Ok(Path::new(file_path).exists())
}

fn print_version() {
    println!(env!("CARGO_PKG_VERSION"));
}

fn init(_args: Vec<String>) {
    if ci_info::is_ci() {
        println!("CI Environment detected. Skipping hook install");
        exit(0);
    }

    if let Err(err) = rusty_hook::init(
        &get_command_runner(),
        &get_file_writer(),
        &get_file_existence_checker(),
    ) {
        eprintln!(
            "Fatal error encountered during initialization. Details: {}",
            err
        );
        exit(1);
    };
}

fn main() {
    let args: Vec<String> = env::args().collect();
    // let program = args[0].clone();
    let first_opt = args[1].clone();
    match first_opt.as_ref() {
        "-v" => print_version(),
        "--version" => print_version(),
        "init" => init(args),
        _ => panic!("Unknown command or option: {}", first_opt),
    };
}
