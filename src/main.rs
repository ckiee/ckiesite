mod parse;

use anyhow::Result;
use combine::{stream::position, EasyParser};
use std::{
    env,
    fs::File,
    io::{self, Read},
};

use crate::parse::pass::{flat_nodes_to_tree, StopAt};

fn main() -> Result<()> {
    match env::args().nth(1) {
        Some(file) => parse_input(File::open(file)?),
        None => parse_input(io::stdin()),
    }
}

fn parse_input(mut read: impl Read) -> Result<()> {
    let buf = Box::leak(Box::new(String::new()));
    read.read_to_string(buf)?;
    let ast = parse::combine::org_file().easy_parse(position::Stream::new(&buf[..]))?.0;
    let tree = flat_nodes_to_tree(&mut ast.iter().peekable(), StopAt::Eof);
    dbg!(&tree);
    Ok(())
}
