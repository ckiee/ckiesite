mod parse;

use anyhow::Result;
use std::{
    env,
    fs::File,
    io::{self, Read},
};

use crate::parse::parse_n_pass;

fn main() -> Result<()> {
    match env::args().nth(1) {
        Some(file) => parse_input(File::open(file)?),
        None => parse_input(io::stdin()),
    }
}

fn parse_input(mut read: impl Read) -> Result<()> {
    let buf = Box::leak(Box::new(String::new()));
    read.read_to_string(buf)?;
    let ast = parse_n_pass(buf);
    dbg!(&ast);
    Ok(())
}
