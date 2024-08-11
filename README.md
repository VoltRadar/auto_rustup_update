# Auto rustup update

This is a small program that checks for new versions of Rust and Rustup.
This can be useful to automaticlly detect if a new patch was just anounced

When it finds an update, it will prompt you to update Rust before
updating.

This README includes instructions to setup this program.

## Requirements

- Ubuntu OS
- `cargo`
- `rustup`
- `git`

If you don't have `cargo` or `rustup` installed, go
[here](https://www.rust-lang.org/tools/install). These tools are used to
complie the program and update it respectivly.

The [Ubuntu](https://ubuntu.com/download) requirement is due to this
programs use of programs installed on Ubuntu which are `systemd`,
`zenity`, and `gnome-terminal`. zenity is used for prompting the user, and
gnome-terminal is used to display a terminal window to display update
status. `systemd` services are the way it runs automaticly.

I think any Linux distrobution that has `systemd`, `zenity`, and
`gnome-terminal` should work. Windows machines will not work without
manual tweaks.

## Setup

1. Clone this repo using the command below:

```
git clone git@github.com:VoltRadar/auto_rustup_update.git
```

2. Navigate into the cloned repository and compile the project using
`cargo build --release`. There now should be a folder called `target`.
Verify that `./target/release/auto_rustup_check` works.

3. Edit the `service/auto_rustup_update.service` file and change the
`ExecStart` path to the path to your location compiled binary. Check the
home directory.

4. Move, or create a soft link, the service files in the `service` directory to the `~/.config/systemd/user` directory. Edit the following commands to create a soft link:

```
ln -s /home/daisy/code/auto_rustup_update/service/auto_rustup_update.timer /home/daisy/.config/systemd/user/auto_rustup_update.timer
ln -s /home/daisy/code/auto_rustup_update/service/auto_rustup_update.service /home/daisy/.config/systemd/user/auto_rustup_update.service

# Edit the first paths to the paths to the service files
# Absolute paths are required, to my understanding
```

5. Start and enable the timer using `systemctl --user start auto_rustup_update.timer` and `systemctl --user start auto_rustup_update.timer`. Verfiy all is working by running `systemctl --user status auto_rustup_update.service`. You should see an output like the one below:

```
○ auto_rustup_update.service - Automatically run rustup check
     Loaded: loaded (/home/daisy/.config/systemd/user/auto_rustup_update.service; linked; preset: enabled)
     Active: inactive (dead) since Sun 2024-08-11 20:04:20 BST; 11min ago
TriggeredBy: ● auto_rustup_update.timer
    Process: 5715 ExecStart=/home/daisy/code/auto_rustup_update/target/release/auto_rustup_update (code=exited, status=0/SUCCESS)
   Main PID: 5715 (code=exited, status=0/SUCCESS)
        CPU: 45ms

Aug 11 20:04:19 ****** systemd[1946]: Starting auto_rustup_update.service - Automatically run rustup check...
Aug 11 20:04:20 ****** auto_rustup_update[5715]: No new updates available
Aug 11 20:04:20 ****** systemd[1946]: Finished auto_rustup_update.service - Automatically run rustup check.
```

That should be it! It will now check for new Rust and Rustup versions
every hour automaticly.