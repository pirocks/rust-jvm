use std::io::{Write};
use std::io;

pub fn prolog_initial_defs(w :&mut dyn Write) -> Result<(),io::Error>{
    write!(w,"['/home/francis/rust-jvm/src/verification/verification.pl'].\n")?;

    Ok(())
}
