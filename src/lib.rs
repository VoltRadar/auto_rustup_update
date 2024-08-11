use std::{collections::HashMap, env, fs, io, os::linux::fs::MetadataExt, path, process, time};

use regex::Regex;

// Path relative to the home path of no-update flag
const RUSTUP_FLAG_PATH: &str = ".rustup/donotupdate";
const RUSTUP_BIN_PATH: &str = ".cargo/bin/rustup";

// Time taken between writing the no-update flag and
const NO_UPDATE_FLAG_DELAY: u64 = 60 * 60 * 24;

// Gets the path to the flag used to set if it should update
fn get_flag_filepath() -> path::PathBuf {
    let mut path = path::PathBuf::new();
    path.push(env::var("HOME").expect("HOME env variable not set!"));
    path.push(RUSTUP_FLAG_PATH);

    return path;
}

fn get_rustup_filepath() -> path::PathBuf {
    let mut path = path::PathBuf::new();
    path.push(env::var("HOME").expect("HOME env variable not set!"));
    path.push(RUSTUP_BIN_PATH);

    return path;
}

fn read_no_update_flag() -> Option<i64> {
    let path = get_flag_filepath();

    match fs::File::open(path) {
        io::Result::Err(error) => {
            if error.kind() == io::ErrorKind::NotFound {
                return None;
            }
        }

        io::Result::Ok(file) => return Some(file.metadata().unwrap().st_mtime()),
    }

    return None;
}

/// Sets the no update flag
///
/// If the argument is true, then set the creation time of the no update
/// flag is updated, or the flag is created
///
/// Else, then the flag is deleted
///
/// Program doesn't prompt for update if the no-update flag is set less
/// then a day ago
fn set_no_update_flag(write_new_flag: bool) -> io::Result<()> {
    let path = get_flag_filepath();

    // Delete the flag
    let result = fs::remove_file(&path);
    if result.is_err() {
        let err = result.err().unwrap();
        match err.kind() {
            io::ErrorKind::NotFound => {}
            _ => return io::Result::Err(err),
        }
    }

    if write_new_flag {
        fs::File::create(&path)?;
    }

    return io::Result::Ok(());
}

/// Returns if the program should prompt the user for an update
///
/// Checks the reboot flag, and returns true if the flag doesn't exist, or
/// is older than 1 day
fn should_prompt() -> bool {
    match read_no_update_flag() {
        None => return true,
        Some(write_time) => {
            // Write time was before 1970, which probably means we should update?
            if write_time < 0 {
                return true;
            }

            let now = time::SystemTime::now()
                .duration_since(time::UNIX_EPOCH)
                .expect("Couldn't compare now to unix epoch")
                .as_secs();
            let diff = now.checked_sub(write_time as u64);

            if diff.is_none() {
                // Creation time of the flag is in the apparent future... should update
                return true;
            }

            let diff = diff.expect("Checked");

            return NO_UPDATE_FLAG_DELAY < diff;
        }
    }
}

/// Run the rustup check command, return a vector of the lines
///
/// Panics on the fail of the command
fn get_rustup_check() -> Vec<String> {

    let mut rustup_path = path::PathBuf::from(env::var("HOME").expect("Home env variable not set!"));
    rustup_path.push(RUSTUP_BIN_PATH);

    let output = process::Command::new(get_rustup_filepath()).arg("check").output();

    if output.is_err() {
        eprintln!("Failed to run rustup!");

        let err = output.err().expect("Checked if error");

        match err.kind() {
            io::ErrorKind::NotFound => {
                panic!("Can't find rustup command. Who is this running as?\n")
            }
            _ => {}
        }

        panic!("{:?}", err);
    }

    let output = output.expect("Checked for error");

    // If it didn't run successfully
    if !output.status.success() {
        let stderr: String =
            String::from_utf8(output.stderr).expect("Failed utf8 decode for std error");

        if stderr.contains("could not download file") {
            panic!("Failed to download file. Check internet connection");
        } else {
            panic!("Unknown error in rustup command!");
        }
    }

    let stdout: String = String::from_utf8(output.stdout).expect("failed utf8 decode for stdout");

    // Split by new lines, filter out empty lines, and clone the lines and
    // collect them into a vector
    return stdout
        .split('\n')
        .filter(|x| x.len() > 0)
        .map(|x| x.to_string())
        .collect();
}

/// Takes the lines from the rustup command and returns the version
/// strings of any new versions of Rust and Rustup
pub fn get_new_versions(rustup_check_lines: Vec<&str>) -> HashMap<&str, Option<&str>> {
    let mut new_versions = HashMap::new();

    let sem_ver_regex = Regex::new(r"[0-9]+\.[0-9]+\.[0-9]+").unwrap();

    for line in rustup_check_lines {
        // Name of toolchain to update
        let name = line
            .split(" - ")
            .next()
            .expect("Rustup output is malformed");

        // No update needed
        if line.contains("Up to date") {
            new_versions.insert(name, None);
        }
        // Updates are needed
        else if line.contains("Update available") {
            // Get the last sem ver string ('1.80.1' and the like) from the rustup line
            let new_version = sem_ver_regex
                .find_iter(line)
                .last()
                .expect("No regex matches")
                .as_str();

            new_versions.insert(name, Some(new_version));
        } else {
            panic!("Rustup line '{line}' is malformed!")
        }
    }

    return new_versions;
}

#[derive(PartialEq, Debug)]
pub enum UpdatePromptAnswer {
    NoUpdateFound,
    Update,
    DoNotUpdate,
    Timeout,
}

/// Analyse the output from the new versions, and prompt the user for an update if needed.
pub fn prompt_for_update(new_versions: HashMap<&str, Option<&str>>) -> UpdatePromptAnswer {
    // Example:

    // zenity --question --title="Rust Update" --no-wrap
    // --text="Rust 1.80.1\nRustup 1.6.0\nUpdate?" --timeout=10 --ok-label="Update"
    // --cancel-label="Not today"

    // Check no new versions were found
    if new_versions.values().all(|new_ver| new_ver.is_none()) {
        return UpdatePromptAnswer::NoUpdateFound;
    }

    let mut args = vec![
        "--question",
        "--title=Rust Update",
        "--no-wrap",
        "--timeout=10",
        "--ok-label=Update",
        "--cancel-label=Not today",
    ];

    // Create --text parameter containing new program versions
    let mut text = String::new();

    for (program, new_version) in new_versions {
        match new_version {
            Some(version) => {
                text.push_str(&format!("{}: {}\n", program, version));
            }
            None => {}
        }
    }

    // Cut new line character
    match text.strip_suffix("\n") {
        Some(stripped_text) => text = stripped_text.to_owned(),
        None => {}
    }

    text = format!("--text={}\nUpdate?", text);
    args.push(&text);

    let prompt_response = process::Command::new("zenity").args(args).spawn();

    if prompt_response.is_err() {
        let error = prompt_response.err().expect("Checked");

        if error.kind() == io::ErrorKind::NotFound {
            panic!("Can't run zenity command. Is zenity installed?");
        } else {
            panic!("Failed to run zenity command due to {:?}", error);
        }
    }

    let prompt_response = prompt_response
        .ok()
        .expect("Checked")
        .wait()
        .expect("Failed to get zenity output");

    match prompt_response.code() {
        Some(0) => return UpdatePromptAnswer::Update,
        Some(1) => return UpdatePromptAnswer::DoNotUpdate,
        Some(5) => return UpdatePromptAnswer::Timeout,
        x => panic!("zenity returned with unexpected error: {:?}", x),
    }
}

pub fn run_update() -> bool {
    let args = [
        "--",
        "/bin/sh",
        "-c",
        "rustup update; echo 'Finished!'; sleep 10",
    ];

    let result = process::Command::new("/bin/gnome-terminal")
        .args(args)
        .output()
        .expect("Update command failed");

    dbg!(&result);

    return result.status.success();
}

/// Main function
///
/// Automaticity checks for new Rust versions prompting user to update
/// Rust. Updates Rust in terminal window if asked. Doesn't ask for a day
/// if told not to update
///
/// Panics if no internet connection
///
/// Panics if couldn't find the `rustup` or `zenity` command
/// 
/// Panics if rustup update doesn't work successfully
pub fn auto_update() -> io::Result<()> {
    let rustup_lines = get_rustup_check();
    let new_versions = get_new_versions(rustup_lines.iter().map(|x| x.as_str()).collect());

    // No new versions
    if new_versions.values().all(|x| x.is_none()) {
        
        // Remove do not update flag
        set_no_update_flag(false)?;

        println!("No new updates available");

        return io::Result::Ok(());
    }

    println!("Updates found:");
    println!("{:?}", new_versions);

    if should_prompt() {
        match prompt_for_update(new_versions) {
            UpdatePromptAnswer::NoUpdateFound => {
                panic!("This should have been handled above")
            }
            UpdatePromptAnswer::DoNotUpdate => {
                println!("User said no updates. Setting no update flag");
                set_no_update_flag(true)?;
            }
            UpdatePromptAnswer::Timeout => {
                println!("Prompt timed out. Asking later...")
            }
            UpdatePromptAnswer::Update => {
                println!("Updated Rust in new terminal");
                if run_update() {
                    println!("Update complete")
                } else {
                    panic!("Update didn't run successfully!")
                }
            }
        }
    } else {
        println!("User said no update in the past... won't prompt for a while")
    }

    return io::Result::Ok(());
}

#[cfg(test)]
mod tests {
    use crate::*;

    #[test]
    fn pass() {
        assert!(true);
    }

    #[test]
    fn rustup_command_test() {
        let rustup_output = get_rustup_check();
        assert_eq!(rustup_output.len(), 2);

        assert!(rustup_output[1].contains("rustup"));
    }

    #[ignore = "Only passes without internet"]
    #[test]
    #[should_panic]
    fn rustup_no_internet() {
        get_rustup_check();
    }

    #[test]
    fn rustup_no_update() {
        let input = vec![
            "stable-x86_64-unknown-linux-gnu - Up to date : 1.80.0 (051478957 2024-07-21)",
            "rustup - Up to date : 1.27.1",
        ];

        let results = get_new_versions(input);

        assert_eq!(results.get("stable-x86_64-unknown-linux-gnu"), Some(&None));
        assert_eq!(results.get("rustup"), Some(&None));
    }

    #[test]
    fn rustup_patch() {
        let input = vec![
            "stable-x86_64-unknown-linux-gnu - Update available : 1.80.0 -> 1.80.1 (051478957 2024-07-21)",
            "rustup - Up to date : 1.27.1",
        ];

        let results = get_new_versions(input);

        assert_eq!(
            results.get("stable-x86_64-unknown-linux-gnu"),
            Some(&Some("1.80.1"))
        );

        assert_eq!(results.get("rustup"), Some(&None));
    }

    #[test]
    fn no_prompt() {
        let mut input: HashMap<&str, Option<&str>> = HashMap::new();
        input.insert("Rust", None);
        input.insert("Rustup", None);

        assert_eq!(prompt_for_update(input), UpdatePromptAnswer::NoUpdateFound);
    }

    #[ignore = "Makes prompt, is annoying"]
    #[test]
    fn prompt_update() {
        let mut input: HashMap<&str, Option<&str>> = HashMap::new();
        input.insert("Rust", Some("1.81.0 Update me!"));
        input.insert("Rustup", Some("1.27.3"));

        assert_eq!(prompt_for_update(input), UpdatePromptAnswer::Update);
    }

    #[ignore = "Makes prompt, is annoying"]
    #[test]
    fn prompt_do_not_update() {
        let mut input: HashMap<&str, Option<&str>> = HashMap::new();
        input.insert("Rust", Some("2.0.0 Don't update me please!!"));
        input.insert("Rustup", None);

        assert_eq!(prompt_for_update(input), UpdatePromptAnswer::DoNotUpdate);
    }

    #[ignore = "Makes prompt, is annoying"]
    #[test]
    fn timeout_prompt() {
        let mut input: HashMap<&str, Option<&str>> = HashMap::new();
        input.insert("Rust", Some("2.0.0 Timeout!!!"));
        input.insert("Rustup", Some("Please don't press a button"));

        assert_eq!(prompt_for_update(input), UpdatePromptAnswer::Timeout);
    }

    #[test]
    fn should_prompt_test() {
        // Based on a flag in the filesystem. Can't be run in parrael with other tests if they modify the

        println!("No flag");
        set_no_update_flag(false).unwrap();
        assert_eq!(should_prompt(), true);

        println!("New flag");
        set_no_update_flag(true).unwrap();
        assert_eq!(should_prompt(), false);

        println!("Second new flag");
        set_no_update_flag(true).unwrap();
        assert_eq!(should_prompt(), false);

        println!("Second no flag");
        set_no_update_flag(false).unwrap();
        assert_eq!(should_prompt(), true);

        println!("All passed");
    }

    #[ignore = "Depends on the file system"]
    #[test]
    fn should_prompt_after_day() {
        // Touch the file so it was modified a day ago
        assert_eq!(should_prompt(), true);
    }

    #[ignore = "Terminal opens, annoying"]
    #[test]
    fn update_test() {
        assert!(run_update())
    }
}

// TODO Change rustup to absolute path (in ~/.cargo)
// TODO Make system file for it
