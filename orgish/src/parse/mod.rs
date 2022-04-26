use anyhow::{anyhow, Result};

mod combiner;
mod data;
mod pass;
#[cfg(test)]
mod test;

pub use self::data::*;

pub fn parse_n_pass(input: &str) -> Result<Vec<AstNode>> {
    use combine::stream::position::Stream;
    use combine::EasyParser;
    use combiner::org_file;

    match org_file().easy_parse(Stream::new(input)) {
        Ok((ast, _)) => Ok(pass::flat_nodes_to_tree(
            &mut ast.iter().peekable(),
            pass::StopAt::Eof,
        )),
        Err(pain) => Err(anyhow!(pain.to_string())),
    }
}
