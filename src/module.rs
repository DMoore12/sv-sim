use crate::var_types::{self, *};
use crate::{parse_comment, LexingError, Token};
use log::{debug, error, trace};
use logos::Lexer;
use std::fmt;

/// SystemVerilog module representation
///
/// Contains a module I/O header, variable assignments, combinational
/// and sequential logic, as well as any constants
#[derive(Default)]
pub struct Module {
    /// Module friendly name
    pub name: String,

    /// Module I/O information
    pub io: ModuleIO,

    /// Module "variables" (wire, reg, etc.)
    pub vars: Vec<Var>,
}

impl fmt::Debug for Module {
    fn fmt(&self, _: &mut std::fmt::Formatter) -> fmt::Result {
        debug!("MODULE: {:?}", self.name);
        format!("{0:?}", self.io);
        for var in self.vars.clone() {
            debug!("VAR: {:?}", var);
        }
        Ok(())
    }
}

/// Parses a module to completion
pub fn parse_module<'source>(lexer: &mut Lexer<'source, Token>) -> Result<Module, LexingError> {
    let mut vars: Vec<Var> = Vec::new();

    let io = match parse_module_io(lexer) {
        Ok(ret) => ret,
        Err(_) => ModuleIO::default(),
    };

    trace!("parsing module");

    while let Some(token) = lexer.next() {
        match token {
            Ok(Token::Wire) => vars.push(parse_module_var(lexer, VarType::Wire)?),
            Ok(Token::Reg) => vars.push(parse_module_var(lexer, VarType::Reg)?),
            Ok(Token::Comment) => match parse_comment(lexer) {
                Ok(_) => (),
                Err(e) => {
                    error!(
                        "unexpected error occurred parsing module comment: '{}'",
                        lexer.slice()
                    );
                    return Err(e);
                }
            },
            Ok(Token::WhiteSpace) => (),
            Ok(Token::EndModule) => break,
            Err(e) => {
                error!(
                    "unexpected error occurred parsing sv file: '{}'",
                    lexer.slice()
                );
                return Err(e);
            }
            // _ => warn!("{:?} not implemented", token.unwrap()),
            _ => (),
        }
    }

    Ok(Module {
        name: io.name.to_owned(),
        io,
        vars,
    })
}

fn parse_module_var<'source>(
    lexer: &mut Lexer<'source, Token>,
    var_type: VarType,
) -> Result<Var, LexingError> {
    let mut width = 1;

    trace!("parsing module variable of type {:?}", var_type);

    while let Some(token) = lexer.next() {
        match token {
            Ok(Token::Word) => match parse_name(lexer) {
                Ok(name) => {
                    return Ok(Var {
                        name,
                        var_type,
                        width,
                        ..Default::default()
                    });
                }
                Err(_) => {
                    error!(
                        "unexpected error occurred parsing module wire name: '{}'",
                        lexer.slice()
                    );
                    break;
                }
            },
            Ok(Token::OpenBracket) => width = var_types::parse_width(lexer)?,
            Ok(Token::Comment) => match crate::parse_comment(lexer) {
                _ => (),
            },
            Ok(Token::WhiteSpace) => (),
            Err(e) => {
                error!(
                    "unexpected error occurred parsing module wire: '{}'",
                    lexer.slice()
                );
                return Err(e);
            }
            _ => (),
        }
    }

    Err(LexingError::ModuleWireNotFound)
}

/// Module I/O information
///
/// Stores all inputs, outputs, and inouts for a given module
#[derive(Default)]
pub struct ModuleIO {
    /// Module name
    pub name: String,

    /// Module inputs
    pub inputs: Vec<Input>,

    /// Module outputs
    pub outputs: Vec<Output>,

    // Module combination input/outputs
    pub inouts: Vec<Inout>,
}

impl fmt::Debug for ModuleIO {
    fn fmt(&self, _: &mut std::fmt::Formatter) -> fmt::Result {
        debug!("MODULE I/O: {:?}", self.name);
        for input in self.inputs.clone() {
            debug!("IO: {:?}", input);
        }
        for output in self.outputs.clone() {
            debug!("IO: {:?}", output);
        }
        for inout in self.inouts.clone() {
            debug!("IO: {:?}", inout);
        }
        Ok(())
    }
}

/// Parses a module I/O block to completion
fn parse_module_io<'source>(lexer: &mut Lexer<'source, Token>) -> Result<ModuleIO, LexingError> {
    #[derive(Default)]
    enum State {
        #[default]
        Name,
        Paren,
        IO,
        Semi,
    }

    let mut state = State::default();
    let mut name = String::default();
    let mut inputs: Vec<Input> = Vec::new();
    let mut outputs: Vec<Output> = Vec::new();
    let mut inouts: Vec<Inout> = Vec::new();

    trace!("parsing module I/O");

    while let Some(token) = lexer.next() {
        match state {
            State::Name => match token {
                Ok(Token::Word) => {
                    name = lexer.slice().to_owned();
                    state = State::Paren;
                }
                Ok(Token::WhiteSpace) => (),
                Ok(Token::Newline) => (),
                Err(e) => {
                    error!(
                        "unexpected error occurred parsing module name: '{}'",
                        lexer.slice()
                    );
                    return Err(e);
                }
                _ => error!("expected module name, got {:?}", token.unwrap()),
            },
            State::Paren => match token {
                Ok(Token::OpenParen) => state = State::IO,
                Ok(Token::WhiteSpace) => (),
                Ok(Token::Newline) => (),
                Err(e) => {
                    error!(
                        "unexpected error occurred parsing module open paren: '{}'",
                        lexer.slice()
                    );
                    return Err(e);
                }
                _ => error!("expected '(', got {:?}", token.unwrap()),
            },
            State::IO => match token {
                Ok(Token::Input) => {
                    match parse_input(lexer) {
                        Ok(var) => inputs.push(var),
                        Err(e) => {
                            error!(
                                "unexpected error occurred parsing module input: '{}'",
                                lexer.slice()
                            );
                            return Err(e);
                        }
                    };
                }
                Ok(Token::Output) => {
                    match parse_output(lexer) {
                        Ok(var) => outputs.push(var),
                        Err(e) => {
                            error!(
                                "unexpected error occurred parsing module output: '{}'",
                                lexer.slice()
                            );
                            return Err(e);
                        }
                    };
                }
                Ok(Token::Inout) => {
                    match parse_inout(lexer) {
                        Ok(var) => inouts.push(var),
                        Err(e) => {
                            error!(
                                "unexpected error occurred parsing module inout: '{}'",
                                lexer.slice()
                            );
                            return Err(e);
                        }
                    };
                }
                Ok(Token::Comment) => match parse_comment(lexer) {
                    _ => (),
                },
                Ok(Token::CloseParen) => state = State::Semi,
                Ok(Token::WhiteSpace) => (),
                Ok(Token::Newline) => (),
                Err(e) => {
                    error!(
                        "unexpected error occurred parsing module: '{}'",
                        lexer.slice()
                    );
                    return Err(e);
                }
                _ => error!("expected I/O declaration or ')', got {:?}", token.unwrap()),
            },
            State::Semi => match token {
                Ok(Token::Semicolon) => break,
                Ok(Token::WhiteSpace) => (),
                Ok(Token::Newline) => (),
                Ok(Token::Comment) => match parse_comment(lexer) {
                    _ => (),
                },
                Err(e) => {
                    error!(
                        "unexpected error occurred parsing module semicolon: '{}'",
                        lexer.slice()
                    );
                    return Err(e);
                }
                _ => error!("expected ';', got {:?}", token.unwrap()),
            },
        };
    }

    Ok(ModuleIO {
        name,
        inputs,
        outputs,
        inouts,
    })
}
