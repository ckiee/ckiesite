use combine::{
    attempt, between, choice, many1, parser,
    parser::char::newline,
    parser::{
        char::{alpha_num, string},
        repeat::many,
        token::token,
    },
    satisfy, skip_many, ParseError, Parser, Stream, StreamOnce,
};

#[derive(PartialEq, Debug)]
pub enum AstNode {
    Codeblock {
        language: String,
        code: String,
    },
    SimpleDirective(String, String),
    Text(String),
    Heading {
        level: u16,
        title: Box<AstNode>,
        nodes: Box<Vec<AstNode>>,
    },
}

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

fn lex<Input, P>(p: P) -> impl Parser<Input, Output = P::Output>
where
    P: Parser<Input>,
    Input: Stream<Token = char>,
    <Input as StreamOnce>::Error: ParseError<
        <Input as StreamOnce>::Token,
        <Input as StreamOnce>::Range,
        <Input as StreamOnce>::Position,
    >,
{
    skip_many(whitespaces())
        .and(p.skip(whitespaces()))
        .map(|x| x.1)
}

fn linespace<Input>() -> impl Parser<Input, Output = ()>
where
    Input: Stream<Token = char>,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
{
    let comment = (token('#'), skip_many(satisfy(|c| c != '\n'))).map(|_| ());
    let skipline = newline().map(|_| ());
    many((whitespaces(), skipline.or(comment)).map(|(_, _)| ()))
}

// fn codeblock<Input>() -> impl Parser<Input, Output = AstNode>
// where
//     Input: Stream<Token = char>,
//     Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
// {
//     (
//         string("#+BEGIN_SRC").skip(whitespaces()),
//         many(alpha_num()),
//     )
//         .map(|(_, language)| AstNode::Codeblock { language, code: String::new() })
//         .message("while parsing codeblock")
// }

fn simple_directive<Input>() -> impl Parser<Input, Output = AstNode>
where
    Input: Stream<Token = char>,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
{
    (
        between(string("#+"), token(':'), many(satisfy(|c| c != ':'))),
        whitespaces(),
        many1(satisfy(|c: char| !c.is_control())),
    )
        .map(|(key, _, value)| AstNode::SimpleDirective(key, value))
        .message("while parsing directive")
}

fn text<Input>() -> impl Parser<Input, Output = AstNode>
where
    Input: Stream<Token = char>,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
{
    many1(satisfy(|c: char| !c.is_control()))
        .map(|x| AstNode::Text(x))
        .message("while parsing text")
}

const STARS: &str = "****************************************************************"; // 64

fn heading_with_level<Input>(level: u16) -> impl Parser<Input, Output = AstNode>
where
    Input: Stream<Token = char>,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
{
    (
        whitespaces(),
        string(&STARS[64 - level as usize..]).skip(whitespace()),
        whitespaces(),
        ast_node_with_heading_level(level).skip(linespace()).map(|x| Box::new(x)),
        many(ast_node_with_heading_level(level + 1).skip(linespace())).map(|x| Box::new(x)),
    )
        .map(move |(_, _, _, title, nodes)| AstNode::Heading {
            level,
            title,
            nodes,
        })
        .message("while parsing heading")
}

parser! {
    #[inline]
    pub fn ast_node_with_heading_level[Input](heading_level: u16)(Input) -> AstNode
    where [ Input:  Stream<Token = char> ]
    {
        choice!(simple_directive(), attempt(heading_with_level(heading_level.clone())), text())
    }
}

pub fn org_file<Input>() -> impl Parser<Input, Output = Vec<AstNode>>
where
    Input: Stream<Token = char>,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
{
    many(ast_node_with_heading_level(1).skip(linespace()))
}
