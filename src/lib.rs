// Logging
use log::{info, warn, error, debug, trace};
use logos::{Logos, Lexer};
use std::num::ParseIntError;

// File reading/writing
use std::fs;
use std::io::{Write, BufReader, BufRead, Error, Lines};

#[derive(Default, Debug, Clone, PartialEq)]
pub enum LexingError {
    InvalidInteger(String),
    #[default]
    NonAsciiCharacter,
}

/// Error type returned by calling lex.slice().parse() to u8
impl From<ParseIntError> for LexingError {
    fn from(err: ParseIntError) -> Self {
        use std::num::IntErrorKind::*;

        match err.kind() {
            PosOverflow | NegOverflow => LexingError::InvalidInteger("overflow error".to_owned()),
            _ => LexingError::InvalidInteger("unknown error".to_owned()),
        }
    }
}

#[derive(Logos, Debug, PartialEq)]
#[logos(error = LexingError)]
#[logos(skip r"[ \t\r\f]+")]
pub enum Token {
    #[token("module")]
    Module,

    #[token("endmodule")]
    EndModule,

    #[token("parameter")]
    Parameter,

    #[token("inout")]
    Inout,

    #[token("input")]
    Input,

    #[token("output")]
    Output,

    #[token("reg")]
    Reg,

    #[token("wire")]
    Wire,

    #[token("assign")]
    Assign,

    #[token("if")]
    If,

    #[token("else")]
    Else,

    #[token("begin")]
    Begin,

    #[token("end")]
    End,

    #[token("posedge")]
    Posedge,

    #[token("negedge")]
    Negedge,

    #[token("timescale")]
    Timescale,

    #[regex(r"\d+[np]s")]
    Time,

    #[token("#")]
    Pound,

    #[token("(")]
    OpenParen,

    #[token(")")]
    CloseParen,

    #[token("[")]
    OpenBracket,

    #[token("]")]
    CloseBracket,

    #[token("{")]
    OpenBrace,

    #[token("}")]
    CloseBrace,

    #[token("==")]
    BEQ,

    #[token("<")]
    BLT,

    #[token(">")]
    BGT,

    #[token("<=")]
    BLTE,

    #[token(">=")]
    BGTE,

    #[token("=")]
    Equals,

    #[token("-")]
    Subtract,

    #[token("+")]
    Add,

    #[token("*")]
    Multiply,

    #[token("/")]
    Divide,

    #[token("?")]
    QMark,

    #[token("!")]
    EMark,

    #[token(":")]
    Colon,

    #[token(";")]
    Semicolon,

    #[token(",")]
    Comma,

    #[token("`")]
    BTick,

    #[token("_")]
    Underscore,

    #[token("@")]
    At,

    #[token("\n")]
    Newline,

    #[regex(r"\d+'b\d+")]
    BinaryValue,

    #[regex(r"\d+'bz")]
    HiZValue,

    #[regex(r"//.*\n")]
    Comment,

    #[regex(r"[a-zA-Z]+")]
    Word,

    #[regex(r"[0-9]+", |lex| lex.slice().parse())]
    Integer(u8),
}

pub struct Sim {
    fpath: std::path::PathBuf,
    contents: String,
}

pub fn read_sv_file(path: &std::path::PathBuf) -> Result<String, Error> {
    let contents = fs::read_to_string(path)?;

    let mut lex = Token::lexer(&contents);

    for line in lex.into_iter() {
        match line {
            Ok(l) => (),
            Err(e) => error!("lexing error: {:?}", e)
        }
    }

    Ok(contents)
}