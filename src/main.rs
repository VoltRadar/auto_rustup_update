use std::io;
use auto_rustup_check;

fn main() -> io::Result<()> {
    auto_rustup_check::auto_update()?;

    return io::Result::Ok(());
}
