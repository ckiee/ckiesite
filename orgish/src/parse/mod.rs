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
        Err(pain) => {
            let pos = pain.position;
            let line_range: usize = 3;

            let org_syntax = SyntaxDefinition::load_from_str(include_str!("../org.sublime-syntax"), true, None)?;
            let ps = {
                let mut b = SyntaxSetBuilder::new();
                b.add(org_syntax);
                b.build()
            };

            let ts = ThemeSet::load_defaults();
            let hl_syntax = ps.find_syntax_by_extension("org").unwrap();
            let mut highlighter = HighlightLines::new(hl_syntax, &ts.themes["base16-ocean.dark"]);

            let src: String = input
                .lines()
                .skip((pos.line as usize) - line_range + 1)
                .take(line_range)
                .enumerate()
                .map(|(idx, line)| {
                    let line_with_ending = format!("{line}\n");
                    let highlighted_line = highlighter.highlight(&line_with_ending, &ps);
                    let term_line = as_24_bit_terminal_escaped(&highlighted_line[..], true);

                    if idx == line_range / 2 {
                        format!("{term_line}----^\n")
                    } else {
                        term_line
                    }
                })
                .collect();

            let pain_str = pain.to_string();
            let mut pain_lines = pain_str.lines();
            let pain_header = pain_lines
                .next()
                .expect("the error message to always have content");
            let clear_color = "\x1b[0m";
            let rest_of_pain = pain_lines.intersperse("\n").collect::<String>();

            Err(anyhow!(format!(
                r#"
{pain_header}:
{src} {clear_color}
{rest_of_pain}"#
            )))
        }
    }
}
