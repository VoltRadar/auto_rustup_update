use std::{
    collections::HashMap,
    hash::Hash,
    io,
    os::unix::process::{CommandExt, ExitStatusExt},
    process::{self, Command},
};

use regex::Regex;

/// Run the rustup command, return a vector of the lines
///
/// Panics on the fail of the command
pub fn get_rustup_check() -> Vec<String> {
    let output = process::Command::new("rustup").arg("check").output();

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
fn get_new_versions(rustup_check_lines: Vec<&str>) -> HashMap<&str, Option<&str>> {
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
    // Example
    // zenity --question --title="Rust Update" --no-wrap --text="Rust 1.80.1\nRustup 1.6.0\nUpdate?" --timeout=10 --ok-label="Update" --cancel-label="Not today"

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

    // Create --text paramter containing new program versions
    let mut text = String::new();
    
    for (program, new_version) in new_versions {
        match new_version {
            Some(version) => {
                text.push_str(&format!("{}: {}\n", program, version));
            },
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

#[cfg(test)]
mod tests {
    use crate::*;

    #[test]
    fn pass() {
        assert!(true);
    }

    #[ignore = "always fails"]
    #[test]
    fn fail() {
        panic!("Oh no!");
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

    #[test]
    fn prompt_update() {
        let mut input: HashMap<&str, Option<&str>> = HashMap::new();
        input.insert("Rust", Some("1.81.0 Update me!"));
        input.insert("Rustup", Some("1.27.3"));

        assert_eq!(prompt_for_update(input), UpdatePromptAnswer::Update);
    }

    #[test]
    fn prompt_do_not_update() {
        let mut input: HashMap<&str, Option<&str>> = HashMap::new();
        input.insert("Rust", Some("2.0.0 Don't update me please!!"));
        input.insert("Rustup", None);

        assert_eq!(prompt_for_update(input), UpdatePromptAnswer::DoNotUpdate);
    }

    #[test]
    fn timeout_prompt() {
        let mut input: HashMap<&str, Option<&str>> = HashMap::new();
        input.insert("Rust", Some("2.0.0 Timeout!!!"));
        input.insert("Rustup", Some("Please don't press a button"));

        assert_eq!(prompt_for_update(input), UpdatePromptAnswer::Timeout);
    }
}
