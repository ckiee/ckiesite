mod parser;

use anyhow::Result;
use combine::{stream::position, EasyParser, Parser, Stream};
use std::{
    env,
    fs::File,
    io::{self, Read},
};

fn main() -> Result<()> {
    match env::args().nth(1) {
        Some(file) => parse_input(File::open(file)?),
        None => parse_input(io::stdin()),
    }
}

fn parse_input(mut read: impl Read) -> Result<()> {
    let mut buf = Box::leak(Box::new(String::new()));
    read.read_to_string(&mut buf)?;
    dbg!(parser::org_file().easy_parse(position::Stream::new(&buf[..]))?);
    Ok(())
}
