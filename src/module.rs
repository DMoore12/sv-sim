use crate::var_types::*;
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
        debug!("Module {:?}", self.name);
        format!("{0:?}", self.io);
        for var in self.vars.clone() {
            debug!("{:?}", var);
        }
        Ok(())
    }
}

/// Parses a module to completion
pub fn parse_module<'source>(lexer: &mut Lexer<'source, Token>) -> Result<Module, LexingError> {
    let mut in_wire = false;
    let mut in_reg = false;
    let mut vars: Vec<Var> = Vec::new();

    trace!("parsing module");

    let io = match parse_module_io(lexer) {
        Ok(ret) => ret,
        Err(_) => ModuleIO::default(),
    };

    while let Some(token) = lexer.next() {
        if in_wire {
            match token {
                Ok(Token::Word) => match parse_name(lexer) {
                    Ok(name) => {
                        vars.push(Var {
                            name,
                            var_type: VarType::Wire,
                            ..Default::default()
                        });
                        in_wire = false;
                    }
                    Err(_) => {
                        error!(
                            "unexpected error occurred parsing module wire name: '{}'",
                            lexer.slice()
                        );
                        in_wire = false;
                    }
                },
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
            continue;
        } else if in_reg {
            match token {
                Ok(Token::Word) => match parse_name(lexer) {
                    Ok(name) => {
                        vars.push(Var {
                            name,
                            var_type: VarType::Reg,
                            ..Default::default()
                        });
                        in_reg = false;
                    }
                    Err(_) => {
                        error!(
                            "unexpected error occurred parsing module reg name: '{}'",
                            lexer.slice()
                        );
                        in_reg = false;
                    }
                },
                Ok(Token::Comment) => match crate::parse_comment(lexer) {
                    _ => (),
                },
                Ok(Token::WhiteSpace) => (),
                Err(e) => {
                    error!(
                        "unexpected error occurred parsing module reg: '{}'",
                        lexer.slice()
                    );
                    return Err(e);
                }
                _ => (),
            }
            continue;
        }

        match token {
            Ok(Token::Wire) => in_wire = true,
            Ok(Token::Reg) => in_reg = true,
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
        debug!("ModuleIO {:?}", self.name);
        for input in self.inputs.clone() {
            debug!("{:?}", input);
        }
        for output in self.outputs.clone() {
            debug!("{:?}", output);
        }
        for inout in self.inouts.clone() {
            debug!("{:?}", inout);
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
