use combine::{
    attempt, between, choice, many1, opaque, optional,
    parser::char::{alpha_num, newline},
    parser::{
        char::string,
        combinator::{no_partial, FnOpaque},
        repeat::{many, take_until},
        token::token,
    },
    satisfy, skip_many,
    stream::position,
    EasyParser, ParseError, Parser, Stream,
};

use super::{
    data::{AstNode, BlockExprNode},
    AbstractSyntaxTree, BlockExprTree, BlockType,
};

fn whitespace<Input>() -> impl Parser<Input, Output = char>
where
    Input: Stream<Token = char>,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
{
    satisfy(|c: char| c.is_whitespace() && c != '\n')
}

fn whitespaces<Input>() -> impl Parser<Input, Output = String>
where
    Input: Stream<Token = char>,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
{
    many(whitespace())
}

fn linespace<Input>() -> impl Parser<Input, Output = AstNode>
where
    Input: Stream<Token = char>,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
{
    let comment = (string("# "), skip_many(satisfy(|c| c != '\n')))
        .map(|_| ())
        .expected("a comment");
    let skipline = newline().map(|_| ());
    skipline
        .or(attempt(comment))
        .map(|_| AstNode::Block(BlockType::Inline, vec![BlockExprNode::Linespace]))
        .message("while parsing linespace")
}

fn directive<Input>() -> impl Parser<Input, Output = AstNode>
where
    Input: Stream<Token = char>,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
{
    (
        between(string("#+"), token(':'), many(satisfy(|c| c != ':'))),
        whitespaces(),
        many1(satisfy(|c: char| !c.is_control())),
    )
        .map(|(key, _, value)| AstNode::Directive(key, value))
        .message("while parsing directive")
}

fn source_block<Input>() -> impl Parser<Input, Output = AstNode>
where
    Input: Stream<Token = char>,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
{
    (
        string("#+BEGIN_SRC"), // #+BEGIN_SRC
        whitespaces(),         //
        many1(alpha_num()),    // rust
        newline(),
        take_until(string("#+END_SRC")), // fn main() {}
        string("#+END_SRC"),             // #+END_SRC
    )
        .map(|(_, _, language, _, code, _)| AstNode::SourceBlock { language, code })
        .message("while parsing source block")
}

fn horiz_rule<Input>() -> impl Parser<Input, Output = AstNode>
where
    Input: Stream<Token = char>,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
{
    string("-----")
        .map(|_| AstNode::HorizRule)
        .message("while parsing horiz_rule")
}

fn block_expr_node<Input>() -> FnOpaque<Input, BlockExprNode>
where
    Input: Stream<Token = char>,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
{
    opaque!(no_partial(
        choice!(
            link(),
            inline_code(),
            attempt(nbsp()),
            bold(),
            italic(),
            underline(),
            strikethrough(),
            char()
        )
        .message("while parsing block_expr_node")
    ))
}

fn ast_block_expr_node<Input>() -> impl Parser<Input, Output = AstNode>
where
    Input: Stream<Token = char>,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
{
    many1(block_expr_node()).map(|v| AstNode::Block(BlockType::Block, v))
}

fn char<Input>() -> impl Parser<Input, Output = BlockExprNode>
where
    Input: Stream<Token = char>,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
{
    satisfy(|c: char| !c.is_control())
        .map(BlockExprNode::Char)
        .message("while parsing char")
}

fn marker_char<Input>(ch: char) -> impl Parser<Input, Output = BlockExprTree>
where
    Input: Stream<Token = char>,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
{
    marker_chars(token(ch), Box::new(move || token(ch)))
}

// Avoid trying to use Copy or Clone https://github.com/Marwes/combine/issues/283#issuecomment-658779127
fn marker_chars<Input, P: Parser<Input>>(start: P, end: Box<dyn Fn() -> P>) -> impl Parser<Input, Output = BlockExprTree>
where
    Input: Stream<Token = char>,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
{
    let end_1 = end();
    let end_2 = end();
    (
        start,
        take_until::<String, _, _>(end_1).flat_map(|s| {
            // HACK ouch ouch ouch
            Ok(many1(block_expr_node())
                .easy_parse(position::Stream::new(&s[..]))
                // this is the except on Result
                // TODO it PANICs. Make it not.
                .expect("In marker_char subparser")
                .0)
        }),
        end_2,
    )
        .map(|(_, v, _)| v)
        .message("while parsing marker_chars")
}

fn inline_code<Input>() -> impl Parser<Input, Output = BlockExprNode>
where
    Input: Stream<Token = char>,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
{
    macro_rules! stupid_marker {
        () => {
            token('=').or(token('~'))
        };
    }
    (
        stupid_marker!(),
        take_until::<String, _, _>(stupid_marker!()),
        stupid_marker!(),
    )
        .map(|(_, c, _)| BlockExprNode::Code(c))
        .message("while parsing inline_code")
}

fn bold<Input>() -> impl Parser<Input, Output = BlockExprNode>
where
    Input: Stream<Token = char>,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
{
    marker_char('*')
        .map(BlockExprNode::Bold)
        .message("while parsing bold")
}

fn italic<Input>() -> impl Parser<Input, Output = BlockExprNode>
where
    Input: Stream<Token = char>,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
{
    marker_char('/')
        .map(BlockExprNode::Italic)
        .message("while parsing italic")
}

fn underline<Input>() -> impl Parser<Input, Output = BlockExprNode>
where
    Input: Stream<Token = char>,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
{
    marker_char('_')
        .map(BlockExprNode::Underline)
        .message("while parsing underline")
}

fn strikethrough<Input>() -> impl Parser<Input, Output = BlockExprNode>
where
    Input: Stream<Token = char>,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
{
    marker_char('+')
        .map(BlockExprNode::Strikethrough)
        .message("while parsing strikethrough")
}


fn nbsp<Input>() -> impl Parser<Input, Output = BlockExprNode>
where
    Input: Stream<Token = char>,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
{
    marker_chars(string("nbsp&"), Box::new(|| string("&nbsp")))
        .map(BlockExprNode::NonbreakingSpace)
        .message("while parsing nbsp")
}


fn link<Input>() -> impl Parser<Input, Output = BlockExprNode>
where
    Input: Stream<Token = char>,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
{
    (
        token('['),                             // [[https://ckie.dev][some /text/ with BENs]]
        token('['),                             // [https://ckie.dev]
        take_until::<String, _, _>(token(']')), // https://ckie.dev
        optional((
            token(']'),
            marker_chars(token('['), Box::new(|| token(']'))),             // [<BET>]
            token(']'),
        )),
    )
        .map(|(_, _, link, maybe_bet)| {
            BlockExprNode::Link(link, maybe_bet.map(|s| s.1).unwrap_or(vec![])) // TODO implement this case properly (rec-parse like marker_char)
        })
        .message("while parsing link")
}

fn heading<Input>() -> impl Parser<Input, Output = AstNode>
where
    Input: Stream<Token = char>,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
{
    (
        whitespaces(),
        many1::<Vec<_>, _, _>(token('*')).map(|x: Vec<_>| x.len()),
        many1::<String, _, _>(whitespace()),
        many1(block_expr_node()),
    )
        .map(|(_, level, _, title)| AstNode::Heading {
            level: level
                .try_into()
                .expect("the header level to be smaller than the maximum value of usize"),
            title,
            children: vec![], // we fill this in later
        })
        .message("while parsing heading")
}

pub fn ast_node<Input>() -> impl Parser<Input, Output = AstNode>
where
    Input: Stream<Token = char>,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
{
    choice!(
        attempt(heading()),
        attempt(source_block()),
        directive(),
        // HACK this attempt() isn't really ideal,
        // but it's worth the exact runtime perf for the code encapsulation for now.
        // (BEN link needs to be parsed before ASN horiz_rule)
        attempt(horiz_rule()),
        ast_block_expr_node()
    )
}

pub fn org_file<Input>() -> impl Parser<Input, Output = AbstractSyntaxTree>
where
    Input: Stream<Token = char>,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
{
    // We have a document like this:
    // |ast_node
    // |ast_node
    // And sometimes there are multiple (2+) newlines:
    // |ast_node
    // |
    // |ast_node
    // This has to be interpreted, while the former case of only one newline must not be.
    // Therefore, it really is:
    // |ast_node
    // |linespace
    // |ast_node
    many::<Vec<_>, _, _>(
        many1::<Vec<_>, _, _>(linespace()).or((ast_node(), linespace()).map(|(a, _)| vec![a])),
    )
    .map(|v| v.into_iter().flatten().collect::<Vec<_>>())
}
