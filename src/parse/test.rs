use anyhow::Result;


use crate::parse::{data::BlockExprNode, parse_n_pass, BlockType};

use super::{
    data::AstNode,
};
#[test]
fn parses_directive() -> Result<()> {
    assert_eq!(
        parse_n_pass("#+TITLE: hello\n")?,
        vec![AstNode::Directive("TITLE".to_string(), "hello".to_string())]
    );
    Ok(())
}

#[test]
fn parses_text() -> Result<()> {
    assert_eq!(
        parse_n_pass("hi world\n")?,
        vec![AstNode::Block(BlockType::Block, vec![
            BlockExprNode::Char('h'),
            BlockExprNode::Char('i'),
            BlockExprNode::Char(' '),
            BlockExprNode::Char('w'),
            BlockExprNode::Char('o'),
            BlockExprNode::Char('r'),
            BlockExprNode::Char('l'),
            BlockExprNode::Char('d'),
        ])]
    );
    Ok(())
}

#[test]
fn parses_empty() -> Result<()> {
    assert_eq!(parse_n_pass("")?, vec![],);
    Ok(())
}

#[test]
fn parses_whitespaces() -> Result<()> {
    eprintln!("ws nl ws nl");
    assert_eq!(
        parse_n_pass(" \n \n")?,
        vec![
            AstNode::Block(BlockType::Block, vec![BlockExprNode::Char(' ')]),
            AstNode::Block(BlockType::Block, vec![BlockExprNode::Char(' ')])
        ],
    );

    eprintln!("just newlines");
    assert_eq!(
        parse_n_pass("\n\n")?,
        vec![]
    );
    Ok(())
}

#[test]
fn parses_comment() -> Result<()> {
    let tests = vec!["\n# test\n", "\n\n# test\n", "# test\n", "# test \n"];
    for test in tests {
        assert!(
            parse_n_pass(test)?
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
            parse_n_pass(test.0)?,
            vec![AstNode::Heading {
                level: test.1,
                children: vec![AstNode::Block(BlockType::Block, vec![
                    BlockExprNode::Char('b'),
                    BlockExprNode::Char('h'),
                ])],
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
        parse_n_pass("* a\n** b\n** c\n* d\n")?,
        vec![
            AstNode::Heading {
                level: 1,
                title: vec![BlockExprNode::Char('a')],
                children: vec![
                    AstNode::Heading {
                        level: 2,
                        title: vec![BlockExprNode::Char('b')],
                        children: vec![]
                    },
                    AstNode::Heading {
                        level: 2,
                        title: vec![BlockExprNode::Char('c')],
                        children: vec![]
                    }
                ]
            },
            AstNode::Heading {
                level: 1,
                title: vec![BlockExprNode::Char('d')],
                children: vec![]
            }
        ]
    );
    Ok(())
}
// TODO parse more BlockExprs: bold, italics, ..
