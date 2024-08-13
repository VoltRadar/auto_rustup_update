// Clippy configurations
#![allow(clippy::needless_return)]

use std::io;

fn main() -> io::Result<()> {
    auto_rustup_update::auto_update()?;

    return io::Result::Ok(());
}
