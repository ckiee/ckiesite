use anyhow::{anyhow, Result};

mod combiner;
mod data;
mod pass1;
mod pass2;
#[cfg(test)]
mod test;

pub use self::data::*;

pub fn parse_n_pass(input: &str) -> Result<BackreferencedAst> {
    use combine::stream::position::Stream;
    use combine::EasyParser;
    use combiner::org_file;

    match org_file().easy_parse(Stream::new(input)) {
        Ok((ast, _)) => Ok(pass2::add_backreferences(
            &mut pass1::flat_nodes_to_tree(&mut ast.iter().peekable(), pass1::StopAt::Eof, None)?
                .into_iter()
                .peekable(),
        )?),
        Err(pain) => Err(anyhow!(pain.to_string())),
    }
}
