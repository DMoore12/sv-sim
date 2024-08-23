// Logging
use log::{info, warn, error, debug, trace};
use logos::{Logos, Lexer};
use core::time;
use std::num::ParseIntError;

// File reading/writing
use std::fs;
use std::io::{Write, BufReader, BufRead, Error, Lines};

#[derive(Default, Debug, Clone, PartialEq)]
pub enum LexingError {
    InvalidInteger(String),
    #[default]
    UnexpectedToken,
    ImproperTimeFormatting,
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

    #[token("always_comb")]
    Comb,

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

    #[regex(r"\d+ns", nanosecond)]
    #[regex(r"\d+ps", picosecond)]
    Time(f64),

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

#[derive(Default, Debug, Clone)]
pub struct Sim {
    fpath: std::path::PathBuf,
    contents: String,
    mods: Vec<Module>,
}

#[derive(Default, Debug, Clone, Copy)]
pub struct Module {
    sim_time: SimTime,
}

#[derive(Debug, Clone, Copy)]
pub struct SimTime {
    n_time: f64,
    d_time: f64,
}

impl Default for SimTime {
    fn default() -> Self {
        Self {
            n_time: 0.000_001,
            d_time: 0.000_000_001,
        }
    }
}

pub fn read_sv_file(path: &std::path::PathBuf) -> Result<String, std::io::Error> {
    Ok(fs::read_to_string(path)?)
}

pub fn parse_sv_file(file_contents: String) -> Result<Module, LexingError> {
    let mut lexer = Token::lexer(file_contents.as_str());

    while let Some(token) = lexer.next() {
        match token {
            Ok(Token::Module) => (),
            Ok(Token::BTick) => {
                match parse_sim_time(&mut lexer) {
                    Ok(val) => info!("sim time found: {:?}", val),
                    Err(e) => return Err(e),
                }
            }
            Err(e) => return Err(e),
            // _ => warn!("{:?} not implemented", token.unwrap()),
            _ => (),
        }
    }

    Ok(Module::default())
}

fn parse_sim_time<'source>(lexer: &mut Lexer<'source, Token>) -> Result<SimTime, LexingError>{
    let mut n = 0.;
    let mut d = 0.;
    let mut timescale_started = false;
    let mut n_found = false;
    let mut n_search = true;

    while let Some(token) = lexer.next() {
        match token {
            Ok(Token::Timescale) => {
                if timescale_started {
                    return Err(LexingError::UnexpectedToken);
                }

                timescale_started = true;
            }
            Ok(Token::Time(val)) => {
                if !timescale_started {
                    return Err(LexingError::UnexpectedToken);
                }

                if n_search {
                    if n_found {
                        return Err(LexingError::ImproperTimeFormatting);
                    }

                    n = val;
                    n_found = true;
                } else {
                    d = val;
                    break;
                }
            },
            Ok(Token::Divide) => n_search = false,
            Err(e) => return Err(e),
            _ => error!("Unexpected token {:?}", token.unwrap()),
        }
    }

    Ok(SimTime {
        n_time: n,
        d_time: d,
    })
}

fn picosecond(lex: &mut Lexer<Token>) -> Option<f64> {
    let slice = lex.slice();
    let n: Result<f64, _> = slice[..slice.len() - 2].parse();

    match n {
        Ok(val) => Some(val * 0.000_000_001),
        Err(e) => { error!("could not read picosecond time: {}", e); None} 
    }
}

fn nanosecond(lex: &mut Lexer<Token>) -> Option<f64> {
    let slice = lex.slice();
    let n: Result<f64, _> = slice[..slice.len() - 2].parse();

    match n {
        Ok(val) => Some(val * 0.000_001),
        Err(e) => { error!("could not read nanosecond time: {}", e); None} 
    }
}
