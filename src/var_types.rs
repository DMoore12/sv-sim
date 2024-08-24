use crate::{LexingError, Token};
use log::{debug, error, info, trace, warn};
use logos::Lexer;
use std::fmt;

#[derive(Default, Debug, Clone)]
pub struct Input {
    pub name: String,
    pub var: Var,
}

#[derive(Default, Debug, Clone)]
pub struct Output {
    pub name: String,
    pub var: Var,
}

#[derive(Default, Debug, Clone)]
pub struct Inout {
    pub name: String,
    pub var: Var,
}

#[derive(Default, Debug, Clone)]
pub enum VarType {
    #[default]
    Wire,
    Reg,
}

impl From<&str> for VarType {
    fn from(val: &str) -> Self {
        match val {
            "wire" => VarType::Wire,
            "reg" => VarType::Reg,
            _ => VarType::default(),
        }
    }
}

#[derive(Default, Debug, Clone)]
pub struct Var {
    pub name: String,
    pub width: u64,
    pub var_type: VarType,
    pub state: bool,
    pub hi_z: bool,
}

pub fn parse_input<'source>(lexer: &mut Lexer<'source, Token>) -> Result<Input, LexingError> {
    trace!("parsing input");

    match parse_var(lexer) {
        Ok((var_type, name, width)) => Ok(Input {
            name: name.to_owned(),
            var: Var {
                name: name.to_owned(),
                width,
                var_type,
                state: false,
                hi_z: false,
            },
        }),
        Err(e) => {
            error!(
                "unexpected error occurred parsing input: '{}'",
                lexer.slice()
            );
            return Err(e);
        }
    }
}

pub fn parse_output<'source>(lexer: &mut Lexer<'source, Token>) -> Result<Output, LexingError> {
    trace!("parsing output");

    match parse_var(lexer) {
        Ok((var_type, name, width)) => Ok(Output {
            name: name.to_owned(),
            var: Var {
                name: name.to_owned(),
                width,
                var_type,
                state: false,
                hi_z: false,
            },
        }),
        Err(e) => {
            error!(
                "unexpected error occurred parsing output: '{}'",
                lexer.slice()
            );
            return Err(e);
        }
    }
}

pub fn parse_inout<'source>(lexer: &mut Lexer<'source, Token>) -> Result<Inout, LexingError> {
    trace!("parsing inout");

    match parse_var(lexer) {
        Ok((var_type, name, width)) => Ok(Inout {
            name: name.to_owned(),
            var: Var {
                name: name.to_owned(),
                width,
                var_type,
                state: false,
                hi_z: false,
            },
        }),
        Err(e) => {
            error!(
                "unexpected error occurred parsing input: '{}'",
                lexer.slice()
            );
            return Err(e);
        }
    }
}

pub fn parse_var<'source>(
    lexer: &mut Lexer<'source, Token>,
) -> Result<(VarType, String, u64), LexingError> {
    let mut width: u64 = 1;
    let mut var_type = VarType::default();

    trace!("parsing variable");

    while let Some(token) = lexer.next() {
        match token {
            Ok(Token::Wire) => var_type = VarType::Wire,
            Ok(Token::Reg) => var_type = VarType::Reg,
            Ok(Token::Word) => match parse_name(lexer) {
                Ok(name) => return Ok((var_type, name, width)),
                Err(e) => return Err(e),
            },
            Ok(Token::OpenBracket) => match parse_width(lexer) {
                Ok(val) => width = val,
                Err(e) => return Err(e),
            },
            Ok(Token::Comment) => match crate::parse_comment(lexer) {
                _ => (),
            },
            Ok(Token::WhiteSpace) => (),
            Err(e) => {
                error!(
                    "unexpected error occurred parsing variable: '{}'",
                    lexer.slice()
                );
                return Err(e);
            }
            _ => error!(
                "unexpected value in variable parsing, got {:?}",
                token.unwrap()
            ),
        }
    }

    Err(LexingError::UnexpectedToken)
}

pub fn parse_name<'source>(lexer: &mut Lexer<'source, Token>) -> Result<String, LexingError> {
    let mut name = lexer.slice().to_owned();

    trace!("parsing variable name");

    while let Some(token) = lexer.next() {
        match token {
            Ok(Token::Word) => name += lexer.slice(),
            Ok(Token::Underscore) => name += "_",
            Ok(Token::WhiteSpace) | Ok(Token::Newline) => (),
            Ok(Token::Semicolon) | Ok(Token::Comma) => return Ok(name),
            Err(e) => {
                error!(
                    "unexpected error occurred parsing variable name: '{}'",
                    lexer.slice()
                );
                return Err(e);
            }
            _ => error!("unexpected value in name parsing, got {:?}", token.unwrap()),
        };
    }

    Ok(name)
}

fn parse_width<'source>(lexer: &mut Lexer<'source, Token>) -> Result<u64, LexingError> {
    let mut start = 0;
    let mut end = 0;
    let mut end_found = false;

    trace!("parsing variable width");

    while let Some(token) = lexer.next() {
        match token {
            Ok(Token::Integer(val)) => {
                if !end_found {
                    end = val;
                    end_found = true;
                } else {
                    start = val;
                }
            }
            Ok(Token::Colon) | Ok(Token::WhiteSpace) => (),
            Ok(Token::CloseBracket) => {
                if end < start {
                    error!(
                        "cannot assign a negative width to var (start: {}, end: {})",
                        start, end
                    );
                    return Err(LexingError::NegativeBitWidth);
                }

                return Ok(end - start + 1);
            }
            Err(e) => {
                error!(
                    "unexpected error occurred parsing variable width: '{}'",
                    lexer.slice()
                );
                return Err(e);
            }
            _ => error!(
                "unexpected value in variable width parsing, got {:?}",
                token.unwrap()
            ),
        }
    }

    Err(LexingError::IncompleteWidth)
}
