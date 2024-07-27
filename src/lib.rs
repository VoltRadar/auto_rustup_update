use std::{io, process};

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

#[cfg(test)]
mod tests {
    use crate::get_rustup_check;

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
    fn rustup_test() {
        let rustup_output = get_rustup_check();
        assert_eq!(rustup_output.len(), 2);

        assert!(rustup_output[1].contains("rustup"));
    }

    #[ignore = "Only passes without internet"]
    #[test]
    #[should_panic]
    fn rustup_no_internet() {
        // Run without internet
        get_rustup_check();
    }
}
