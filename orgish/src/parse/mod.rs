use anyhow::Result;

mod combiner;
mod data;
mod pass;
#[cfg(test)]
mod test;

pub use data::*;

pub fn parse_n_pass(input: &'static str) -> Result<Vec<data::AstNode>> {
    use combine::stream::position::Stream;
    use combine::EasyParser;
    use combiner::org_file;

    let ast = org_file().easy_parse(Stream::new(input))?.0;
    Ok(pass::flat_nodes_to_tree(
        &mut ast.iter().peekable(),
        pass::StopAt::Eof,
    ))
}
