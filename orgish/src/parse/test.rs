use anyhow::Result;

use anyhow::anyhow;
use crate::parse::BackrefAstNode;
use crate::parse::{data::BlockExprNode, BlockType, Directive, LinkTarget, Route};

use super::AbstractSyntaxTree;
use super::data::AstNode;

pub fn parse_partial_pass(input: &str) -> Result<AbstractSyntaxTree> {
    use combine::stream::position::Stream;
    use combine::EasyParser;
    use super::combiner::org_file;

    match org_file().easy_parse(Stream::new(input)) {
        Ok((ast, _)) => Ok(super::pass1::flat_nodes_to_tree(
            &mut ast.iter().peekable(),
            super::pass1::StopAt::Eof,
        )?),
        Err(pain) => Err(anyhow!(pain.to_string())),
    }
}

#[test]
fn parses_directive() -> Result<()> {
    assert_eq!(
        parse_partial_pass("#+TITLE: hello\n")?,
        vec![AstNode::Directive(Directive::Title("hello".to_string()))]
    );
    // TODO add more directiev types
    Ok(())
}

#[test]
fn parses_text() -> Result<()> {
    assert_eq!(
        parse_partial_pass("hi world\n")?,
        vec![AstNode::Block(
            BlockType::Block,
            vec![
                BlockExprNode::Char('h'),
                BlockExprNode::Char('i'),
                BlockExprNode::Char(' '),
                BlockExprNode::Char('w'),
                BlockExprNode::Char('o'),
                BlockExprNode::Char('r'),
                BlockExprNode::Char('l'),
                BlockExprNode::Char('d'),
            ]
        )]
    );
    Ok(())
}

#[test]
fn parses_empty() -> Result<()> {
    assert_eq!(parse_partial_pass("")?, vec![],);
    Ok(())
}

#[test]
fn parses_whitespaces() -> Result<()> {
    eprintln!("ws nl ws nl");
    assert_eq!(
        parse_partial_pass(" \n \n")?,
        vec![
            AstNode::Block(BlockType::Block, vec![BlockExprNode::Char(' ')]),
            AstNode::Block(BlockType::Block, vec![BlockExprNode::Char(' ')])
        ],
    );

    eprintln!("just newlines");
    assert_eq!(parse_partial_pass("\n\n")?, vec![]);
    Ok(())
}

#[test]
fn parses_comment() -> Result<()> {
    let tests = vec!["\n# test\n", "\n\n# test\n", "# test\n", "# test \n"];
    for test in tests {
        assert!(
            parse_partial_pass(test)?
                .iter()
                .all(|n| n == &AstNode::Block(BlockType::Block, vec![BlockExprNode::Linespace])),
            "input: '{}' did not parse to single linespace",
            test
        );
    }
    Ok(())
}

#[test]
fn parses_heading() -> Result<()> {
    struct TI(&'static str, u16);
    let tests = vec![
        TI("* ta\nbh\n", 1),
        TI(" * ta\nbh\n", 1),
        TI("** ta\nbh\n", 2),
    ];
    for test in tests {
        assert_eq!(
            parse_partial_pass(test.0)?,
            vec![AstNode::Heading {
                routing: None,
                level: test.1,
                children: vec![BackrefAstNode::new_unref(AstNode::Block(
                    BlockType::Block,
                    vec![BlockExprNode::Char('b'), BlockExprNode::Char('h'),]
                ))],
                title: vec![BlockExprNode::Char('t'), BlockExprNode::Char('a'),]
            }],
            "\ninput: {} (lvl={})",
            test.0,
            test.1
        );
    }
    Ok(())
}

// Consider three headings:
// * a
// ** b
// ** c
// * d
// `c`/`d` should not be children of `b`
#[test]
fn parse_escapes_heading_level() -> Result<()> {
    assert_eq!(
        parse_partial_pass("* a\n** b\n** c\n* d\n")?,
        vec![
            AstNode::Heading {
                routing: None,
                level: 1,
                title: vec![BlockExprNode::Char('a')],
                children: vec![
                    BackrefAstNode::new_unref(AstNode::Heading {
                        routing: None,
                        level: 2,
                        title: vec![BlockExprNode::Char('b')],
                        children: vec![]
                    }),
                    BackrefAstNode::new_unref(AstNode::Heading {
                        routing: None,
                        level: 2,
                        title: vec![BlockExprNode::Char('c')],
                        children: vec![]
                    })
                ]
            },
            AstNode::Heading {
                routing: None,
                level: 1,
                title: vec![BlockExprNode::Char('d')],
                children: vec![]
            }
        ]
    );
    Ok(())
}

#[test]
fn parses_blockexpr_formatting() -> Result<()> {
    assert_eq!(
        parse_partial_pass(
            r#"To markup text in Org, simply surround it with one or more marker characters. *Bold*, /italic/ and _underline_ are fairly intuitive, and the ability to use +strikethrough+ is a plus.  You can _/*combine*/_ the basic markup in any order, however ~code~ and =verbatim= need to be the *_~inner-most~_* markers if they are present since their contents are interpreted =_literally_=.
"#
        )?,
        vec![AstNode::Block(
            BlockType::Block,
            vec![
                BlockExprNode::Char('T'),
                BlockExprNode::Char('o'),
                BlockExprNode::Char(' '),
                BlockExprNode::Char('m'),
                BlockExprNode::Char('a'),
                BlockExprNode::Char('r'),
                BlockExprNode::Char('k'),
                BlockExprNode::Char('u'),
                BlockExprNode::Char('p'),
                BlockExprNode::Char(' '),
                BlockExprNode::Char('t'),
                BlockExprNode::Char('e'),
                BlockExprNode::Char('x'),
                BlockExprNode::Char('t'),
                BlockExprNode::Char(' '),
                BlockExprNode::Char('i'),
                BlockExprNode::Char('n'),
                BlockExprNode::Char(' '),
                BlockExprNode::Char('O'),
                BlockExprNode::Char('r'),
                BlockExprNode::Char('g'),
                BlockExprNode::Char(','),
                BlockExprNode::Char(' '),
                BlockExprNode::Char('s'),
                BlockExprNode::Char('i'),
                BlockExprNode::Char('m'),
                BlockExprNode::Char('p'),
                BlockExprNode::Char('l'),
                BlockExprNode::Char('y'),
                BlockExprNode::Char(' '),
                BlockExprNode::Char('s'),
                BlockExprNode::Char('u'),
                BlockExprNode::Char('r'),
                BlockExprNode::Char('r'),
                BlockExprNode::Char('o'),
                BlockExprNode::Char('u'),
                BlockExprNode::Char('n'),
                BlockExprNode::Char('d'),
                BlockExprNode::Char(' '),
                BlockExprNode::Char('i'),
                BlockExprNode::Char('t'),
                BlockExprNode::Char(' '),
                BlockExprNode::Char('w'),
                BlockExprNode::Char('i'),
                BlockExprNode::Char('t'),
                BlockExprNode::Char('h'),
                BlockExprNode::Char(' '),
                BlockExprNode::Char('o'),
                BlockExprNode::Char('n'),
                BlockExprNode::Char('e'),
                BlockExprNode::Char(' '),
                BlockExprNode::Char('o'),
                BlockExprNode::Char('r'),
                BlockExprNode::Char(' '),
                BlockExprNode::Char('m'),
                BlockExprNode::Char('o'),
                BlockExprNode::Char('r'),
                BlockExprNode::Char('e'),
                BlockExprNode::Char(' '),
                BlockExprNode::Char('m'),
                BlockExprNode::Char('a'),
                BlockExprNode::Char('r'),
                BlockExprNode::Char('k'),
                BlockExprNode::Char('e'),
                BlockExprNode::Char('r'),
                BlockExprNode::Char(' '),
                BlockExprNode::Char('c'),
                BlockExprNode::Char('h'),
                BlockExprNode::Char('a'),
                BlockExprNode::Char('r'),
                BlockExprNode::Char('a'),
                BlockExprNode::Char('c'),
                BlockExprNode::Char('t'),
                BlockExprNode::Char('e'),
                BlockExprNode::Char('r'),
                BlockExprNode::Char('s'),
                BlockExprNode::Char('.'),
                BlockExprNode::Char(' '),
                BlockExprNode::Bold(vec![
                    BlockExprNode::Char('B'),
                    BlockExprNode::Char('o'),
                    BlockExprNode::Char('l'),
                    BlockExprNode::Char('d')
                ]),
                BlockExprNode::Char(','),
                BlockExprNode::Char(' '),
                BlockExprNode::Italic(vec![
                    BlockExprNode::Char('i'),
                    BlockExprNode::Char('t'),
                    BlockExprNode::Char('a'),
                    BlockExprNode::Char('l'),
                    BlockExprNode::Char('i'),
                    BlockExprNode::Char('c')
                ]),
                BlockExprNode::Char(' '),
                BlockExprNode::Char('a'),
                BlockExprNode::Char('n'),
                BlockExprNode::Char('d'),
                BlockExprNode::Char(' '),
                BlockExprNode::Underline(vec![
                    BlockExprNode::Char('u'),
                    BlockExprNode::Char('n'),
                    BlockExprNode::Char('d'),
                    BlockExprNode::Char('e'),
                    BlockExprNode::Char('r'),
                    BlockExprNode::Char('l'),
                    BlockExprNode::Char('i'),
                    BlockExprNode::Char('n'),
                    BlockExprNode::Char('e')
                ]),
                BlockExprNode::Char(' '),
                BlockExprNode::Char('a'),
                BlockExprNode::Char('r'),
                BlockExprNode::Char('e'),
                BlockExprNode::Char(' '),
                BlockExprNode::Char('f'),
                BlockExprNode::Char('a'),
                BlockExprNode::Char('i'),
                BlockExprNode::Char('r'),
                BlockExprNode::Char('l'),
                BlockExprNode::Char('y'),
                BlockExprNode::Char(' '),
                BlockExprNode::Char('i'),
                BlockExprNode::Char('n'),
                BlockExprNode::Char('t'),
                BlockExprNode::Char('u'),
                BlockExprNode::Char('i'),
                BlockExprNode::Char('t'),
                BlockExprNode::Char('i'),
                BlockExprNode::Char('v'),
                BlockExprNode::Char('e'),
                BlockExprNode::Char(','),
                BlockExprNode::Char(' '),
                BlockExprNode::Char('a'),
                BlockExprNode::Char('n'),
                BlockExprNode::Char('d'),
                BlockExprNode::Char(' '),
                BlockExprNode::Char('t'),
                BlockExprNode::Char('h'),
                BlockExprNode::Char('e'),
                BlockExprNode::Char(' '),
                BlockExprNode::Char('a'),
                BlockExprNode::Char('b'),
                BlockExprNode::Char('i'),
                BlockExprNode::Char('l'),
                BlockExprNode::Char('i'),
                BlockExprNode::Char('t'),
                BlockExprNode::Char('y'),
                BlockExprNode::Char(' '),
                BlockExprNode::Char('t'),
                BlockExprNode::Char('o'),
                BlockExprNode::Char(' '),
                BlockExprNode::Char('u'),
                BlockExprNode::Char('s'),
                BlockExprNode::Char('e'),
                BlockExprNode::Char(' '),
                BlockExprNode::Strikethrough(vec![
                    BlockExprNode::Char('s'),
                    BlockExprNode::Char('t'),
                    BlockExprNode::Char('r'),
                    BlockExprNode::Char('i'),
                    BlockExprNode::Char('k'),
                    BlockExprNode::Char('e'),
                    BlockExprNode::Char('t'),
                    BlockExprNode::Char('h'),
                    BlockExprNode::Char('r'),
                    BlockExprNode::Char('o'),
                    BlockExprNode::Char('u'),
                    BlockExprNode::Char('g'),
                    BlockExprNode::Char('h')
                ]),
                BlockExprNode::Char(' '),
                BlockExprNode::Char('i'),
                BlockExprNode::Char('s'),
                BlockExprNode::Char(' '),
                BlockExprNode::Char('a'),
                BlockExprNode::Char(' '),
                BlockExprNode::Char('p'),
                BlockExprNode::Char('l'),
                BlockExprNode::Char('u'),
                BlockExprNode::Char('s'),
                BlockExprNode::Char('.'),
                BlockExprNode::Char(' '),
                BlockExprNode::Char(' '),
                BlockExprNode::Char('Y'),
                BlockExprNode::Char('o'),
                BlockExprNode::Char('u'),
                BlockExprNode::Char(' '),
                BlockExprNode::Char('c'),
                BlockExprNode::Char('a'),
                BlockExprNode::Char('n'),
                BlockExprNode::Char(' '),
                BlockExprNode::Underline(vec![BlockExprNode::Italic(vec![BlockExprNode::Bold(
                    vec![
                        BlockExprNode::Char('c'),
                        BlockExprNode::Char('o'),
                        BlockExprNode::Char('m'),
                        BlockExprNode::Char('b'),
                        BlockExprNode::Char('i'),
                        BlockExprNode::Char('n'),
                        BlockExprNode::Char('e')
                    ]
                )])]),
                BlockExprNode::Char(' '),
                BlockExprNode::Char('t'),
                BlockExprNode::Char('h'),
                BlockExprNode::Char('e'),
                BlockExprNode::Char(' '),
                BlockExprNode::Char('b'),
                BlockExprNode::Char('a'),
                BlockExprNode::Char('s'),
                BlockExprNode::Char('i'),
                BlockExprNode::Char('c'),
                BlockExprNode::Char(' '),
                BlockExprNode::Char('m'),
                BlockExprNode::Char('a'),
                BlockExprNode::Char('r'),
                BlockExprNode::Char('k'),
                BlockExprNode::Char('u'),
                BlockExprNode::Char('p'),
                BlockExprNode::Char(' '),
                BlockExprNode::Char('i'),
                BlockExprNode::Char('n'),
                BlockExprNode::Char(' '),
                BlockExprNode::Char('a'),
                BlockExprNode::Char('n'),
                BlockExprNode::Char('y'),
                BlockExprNode::Char(' '),
                BlockExprNode::Char('o'),
                BlockExprNode::Char('r'),
                BlockExprNode::Char('d'),
                BlockExprNode::Char('e'),
                BlockExprNode::Char('r'),
                BlockExprNode::Char(','),
                BlockExprNode::Char(' '),
                BlockExprNode::Char('h'),
                BlockExprNode::Char('o'),
                BlockExprNode::Char('w'),
                BlockExprNode::Char('e'),
                BlockExprNode::Char('v'),
                BlockExprNode::Char('e'),
                BlockExprNode::Char('r'),
                BlockExprNode::Char(' '),
                BlockExprNode::Code("code".to_string()),
                BlockExprNode::Char(' '),
                BlockExprNode::Char('a'),
                BlockExprNode::Char('n'),
                BlockExprNode::Char('d'),
                BlockExprNode::Char(' '),
                BlockExprNode::Code("verbatim".to_string()),
                BlockExprNode::Char(' '),
                BlockExprNode::Char('n'),
                BlockExprNode::Char('e'),
                BlockExprNode::Char('e'),
                BlockExprNode::Char('d'),
                BlockExprNode::Char(' '),
                BlockExprNode::Char('t'),
                BlockExprNode::Char('o'),
                BlockExprNode::Char(' '),
                BlockExprNode::Char('b'),
                BlockExprNode::Char('e'),
                BlockExprNode::Char(' '),
                BlockExprNode::Char('t'),
                BlockExprNode::Char('h'),
                BlockExprNode::Char('e'),
                BlockExprNode::Char(' '),
                BlockExprNode::Bold(vec![BlockExprNode::Underline(vec![BlockExprNode::Code(
                    "inner-most".to_string()
                )])]),
                BlockExprNode::Char(' '),
                BlockExprNode::Char('m'),
                BlockExprNode::Char('a'),
                BlockExprNode::Char('r'),
                BlockExprNode::Char('k'),
                BlockExprNode::Char('e'),
                BlockExprNode::Char('r'),
                BlockExprNode::Char('s'),
                BlockExprNode::Char(' '),
                BlockExprNode::Char('i'),
                BlockExprNode::Char('f'),
                BlockExprNode::Char(' '),
                BlockExprNode::Char('t'),
                BlockExprNode::Char('h'),
                BlockExprNode::Char('e'),
                BlockExprNode::Char('y'),
                BlockExprNode::Char(' '),
                BlockExprNode::Char('a'),
                BlockExprNode::Char('r'),
                BlockExprNode::Char('e'),
                BlockExprNode::Char(' '),
                BlockExprNode::Char('p'),
                BlockExprNode::Char('r'),
                BlockExprNode::Char('e'),
                BlockExprNode::Char('s'),
                BlockExprNode::Char('e'),
                BlockExprNode::Char('n'),
                BlockExprNode::Char('t'),
                BlockExprNode::Char(' '),
                BlockExprNode::Char('s'),
                BlockExprNode::Char('i'),
                BlockExprNode::Char('n'),
                BlockExprNode::Char('c'),
                BlockExprNode::Char('e'),
                BlockExprNode::Char(' '),
                BlockExprNode::Char('t'),
                BlockExprNode::Char('h'),
                BlockExprNode::Char('e'),
                BlockExprNode::Char('i'),
                BlockExprNode::Char('r'),
                BlockExprNode::Char(' '),
                BlockExprNode::Char('c'),
                BlockExprNode::Char('o'),
                BlockExprNode::Char('n'),
                BlockExprNode::Char('t'),
                BlockExprNode::Char('e'),
                BlockExprNode::Char('n'),
                BlockExprNode::Char('t'),
                BlockExprNode::Char('s'),
                BlockExprNode::Char(' '),
                BlockExprNode::Char('a'),
                BlockExprNode::Char('r'),
                BlockExprNode::Char('e'),
                BlockExprNode::Char(' '),
                BlockExprNode::Char('i'),
                BlockExprNode::Char('n'),
                BlockExprNode::Char('t'),
                BlockExprNode::Char('e'),
                BlockExprNode::Char('r'),
                BlockExprNode::Char('p'),
                BlockExprNode::Char('r'),
                BlockExprNode::Char('e'),
                BlockExprNode::Char('t'),
                BlockExprNode::Char('e'),
                BlockExprNode::Char('d'),
                BlockExprNode::Char(' '),
                BlockExprNode::Code("_literally_".to_string()),
                BlockExprNode::Char('.')
            ]
        )]
    );
    Ok(())
}

#[test]
fn parses_nbsp() -> Result<()> {
    let sp_char = BlockExprNode::Char(' ');
    let nbsp_char = BlockExprNode::Char('\u{a0}');
    assert_eq!(
        parse_partial_pass("nbsp&h h&nbsp\n")?,
        vec![AstNode::Block(
            BlockType::Block,
            vec![
                BlockExprNode::Char('h'),
                nbsp_char,
                BlockExprNode::Char('h')
            ]
        )]
    );
    assert_eq!(
        parse_partial_pass("h h\n")?,
        vec![AstNode::Block(
            BlockType::Block,
            vec![BlockExprNode::Char('h'), sp_char, BlockExprNode::Char('h')]
        )]
    );
    Ok(())
}

#[test]
fn parses_link() -> Result<()> {
    assert_eq!(
        parse_partial_pass("[[https://example.com/some-path][helo]]\n")?,
        vec![AstNode::Block(
            BlockType::Block,
            vec![BlockExprNode::Link(
                LinkTarget::External("https://example.com/some-path".to_string()),
                Some(vec![
                    BlockExprNode::Char('h'),
                    BlockExprNode::Char('e'),
                    BlockExprNode::Char('l'),
                    BlockExprNode::Char('o')
                ])
            )]
        )]
    );

    assert_eq!(
        parse_partial_pass("[[https://example.com/some-path]]\n")?,
        vec![AstNode::Block(
            BlockType::Block,
            vec![BlockExprNode::Link(
                LinkTarget::External("https://example.com/some-path".to_string()),
                None
            )]
        )]
    );

    // rust is not in the mood for proper formatting so this is handmade )^:
    assert_eq!(
        parse_partial_pass("in [[https://github.com/ckiee/nixfiles/blob/master/modules/services/mailserver/util.nix][a lot of places]].\n")?,
        vec![AstNode::Block(BlockType::Block,
            vec![
                BlockExprNode::Char('i'),
                BlockExprNode::Char('n'),
                BlockExprNode::Char(' '),
                BlockExprNode::Link(
                    LinkTarget::External("https://github.com/ckiee/nixfiles/blob/master/modules/services/mailserver/util.nix".to_string()),
                    Some(vec![
                        BlockExprNode::Char('a'),
                        BlockExprNode::Char(' '),
                        BlockExprNode::Char('l'),
                        BlockExprNode::Char('o'),
                        BlockExprNode::Char('t'),
                        BlockExprNode::Char(' '),
                        BlockExprNode::Char('o'),
                        BlockExprNode::Char('f'),
                        BlockExprNode::Char(' '),
                        BlockExprNode::Char('p'),
                        BlockExprNode::Char('l'),
                        BlockExprNode::Char('a'),
                        BlockExprNode::Char('c'),
                        BlockExprNode::Char('e'),
                        BlockExprNode::Char('s')]
                    )
                ),
                BlockExprNode::Char('.')
            ]
        )]
    );

    Ok(())
}

#[test]
fn parses_routing() -> Result<()> {
    assert_eq!(
        parse_partial_pass("* :/path: h\n")?,
        vec![AstNode::Heading {
            level: 1,
            children: vec![],
            title: vec![BlockExprNode::Char('h')],
            routing: Some(Route::Page("/path".to_string()))
        }],
        "\nensuring heading with routing parses with whitespaces"
    );
    assert_eq!(
        parse_partial_pass("*:/path:h\n")?,
        vec![AstNode::Heading {
            level: 1,
            children: vec![],
            title: vec![BlockExprNode::Char('h')],
            routing: Some(Route::Page("/path".to_string()))
        }],
        "\nensuring heading with routing parses without whitespace"
    );
    Ok(())
}

// TODO test internal links
