use std::{collections::HashMap, process};

use auto_rustup_check;

use regex::Regex;

// Little function to test regexs
fn regex_testing() {
    let hey = "stable-x86_64-unknown-linux-gnu - Update available : 1.80.0 -> 1.80.1 (051478957 2024-07-21)";

    // The regex r"(\w+)+" only captures the first occurrence of the group
    // '\w+' it seems. Best to get multiple matches via captures_iter...
    // or find_iter.
    let sem_ver_regex = Regex::new(r"[0-9]+\.[0-9]+\.[0-9]+").unwrap();

    for (version, []) in sem_ver_regex.captures_iter(hey).map(|cap| cap.extract()) {
        dbg!(version);
    }
}

fn prompt_experiment() {
    let mut input: HashMap<&str, Option<&str>> = HashMap::new();
    input.insert("Rust", Some("1.81.0 Update me!"));
    input.insert("Rustup", Some("1.27.3"));

    assert_eq!(
        auto_rustup_check::prompt_for_update(input),
        auto_rustup_check::UpdatePromptAnswer::Update
    );
}

fn spawner_test() {
    let mut args = vec!["--question", "--timeout=10"];

    let mut handle = process::Command::new("zenity").args(args).spawn().unwrap();

    let exit_code = handle.wait().unwrap().code();

    dbg!(&exit_code);
}

fn main() {
    spawner_test();
}
