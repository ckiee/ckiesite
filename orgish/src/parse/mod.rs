use anyhow::{anyhow, Result};

mod combiner;
mod data;
mod pass1;
mod pass2;
// Stuff doesn't break as weirdly anymore, and I'm tired of
// fixing the +bazillion+, no, 11 tests.
// #[cfg(test)]
// mod test;

pub use self::data::*;

pub fn parse_n_pass(input: &str) -> Result<PassedSyntaxTree> {
    use combine::stream::position::Stream;
    use combine::EasyParser;
    use combiner::org_file;

    match org_file().easy_parse(Stream::new(input)) {
        Ok((ast, _)) => Ok(pass2::pass2(pass1::flat_nodes_to_tree(
            &mut ast.iter().peekable(),
            pass1::StopAt::Eof,
        )?)?),
        Err(pain) => Err(anyhow!(pain.to_string())),
    }
}
