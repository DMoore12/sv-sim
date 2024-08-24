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
    pub var_type: VarType,
    pub state: bool,
    pub hi_z: bool,
}

pub fn parse_input<'source>(lexer: &mut Lexer<'source, Token>) -> Result<Input, LexingError> {
    match parse_var(lexer) {
        Ok((var_type, name)) => Ok(Input {
            name: name.to_owned(),
            var: Var {
                name: name.to_owned(),
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
    match parse_var(lexer) {
        Ok((var_type, name)) => Ok(Output {
            name: name.to_owned(),
            var: Var {
                name: name.to_owned(),
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
    match parse_var(lexer) {
        Ok((var_type, name)) => Ok(Inout {
            name: name.to_owned(),
            var: Var {
                name: name.to_owned(),
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
) -> Result<(VarType, String), LexingError> {
    let mut var_type = VarType::default();

    while let Some(token) = lexer.next() {
        match token {
            Ok(Token::Wire) => var_type = VarType::Wire,
            Ok(Token::Reg) => var_type = VarType::Reg,
            Ok(Token::Word) => match parse_name(lexer) {
                Ok(name) => return Ok((var_type, name)),
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

    while let Some(token) = lexer.next() {
        match token {
            Ok(Token::Word) => name += lexer.slice(),
            Ok(Token::Underscore) => name += "_",
            Ok(Token::WhiteSpace)
            | Ok(Token::Newline)
            | Ok(Token::Semicolon)
            | Ok(Token::Comma) => return Ok(name),
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
