// Logging
use log::{debug, error, trace, warn};
use logos::{Lexer, Logos};
use std::num::ParseIntError;

// File reading/writing
use std::fs;

// Debug
use std::fmt;

pub mod var_types;
use var_types::*;

/// Errors occurring due to incorrect character sequences
#[derive(Default, Debug, Clone, PartialEq)]
pub enum LexingError {
    /// Invalid integer found
    InvalidInteger(String),

    /// Unexpected token or generic error
    #[default]
    UnexpectedToken,

    /// Improper time format found
    ImproperTimeFormatting,

    /// Non ASCII character found (not currently used)
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
#[logos(skip r"[\r\f]+")]
pub enum Token {
    /// Module start
    #[token("module")]
    Module,

    /// Module end
    #[token("endmodule")]
    EndModule,

    /// Parameter start
    #[token("parameter")]
    Parameter,

    /// Inout start
    #[token("inout")]
    Inout,

    /// Input start
    #[token("input")]
    Input,

    /// Output start
    #[token("output")]
    Output,

    /// Register start
    #[token("reg")]
    Reg,

    /// Wire start
    #[token("wire")]
    Wire,

    /// Assign start
    #[token("assign")]
    Assign,

    /// Combinational logic start
    #[token("always_comb")]
    Comb,

    /// If statement start
    #[token("if")]
    If,

    /// Else statement start
    #[token("else")]
    Else,

    /// Begin statement
    #[token("begin")]
    Begin,

    /// End statement
    #[token("end")]
    End,

    /// Rising edge trigger
    #[token("posedge")]
    Posedge,

    /// Negative edge trigger
    #[token("negedge")]
    Negedge,

    /// Timescale start
    #[token("timescale")]
    Timescale,

    /// Simulation time
    ///
    /// Accepts times in ns or ps
    #[regex(r"\d+ns", nanosecond)]
    #[regex(r"\d+ps", picosecond)]
    Time(f64),

    /// Pound symbol
    #[token("#")]
    Pound,

    /// Open parenthesis
    #[token("(")]
    OpenParen,

    /// Close parenthesis
    #[token(")")]
    CloseParen,

    /// Open bracket
    #[token("[")]
    OpenBracket,

    /// Close bracket
    #[token("]")]
    CloseBracket,

    /// Open brace
    #[token("{")]
    OpenBrace,

    /// Close brace
    #[token("}")]
    CloseBrace,

    /// Equivalent comparison
    #[token("==")]
    BEQ,

    /// Less than comparison
    #[token("<")]
    BLT,

    /// Greater than comparison
    #[token(">")]
    BGT,

    /// Less than or equal to comparison
    #[token("<=")]
    BLTE,

    /// Greater than or equal to comparison
    #[token(">=")]
    BGTE,

    /// Assignment start
    #[token("=")]
    Equals,

    /// Subtraction
    #[token("-")]
    Subtract,

    /// Addition
    #[token("+")]
    Add,

    /// Multiply
    #[token("*")]
    Multiply,

    /// Divide
    #[token("/")]
    Divide,

    /// Question mark
    #[token("?")]
    QMark,

    /// Exclamation point
    #[token("!")]
    EMark,

    /// Colon
    #[token(":")]
    Colon,

    /// Semicolon
    #[token(";")]
    Semicolon,

    /// Comma
    #[token(",")]
    Comma,

    /// Back tick
    #[token("`")]
    BTick,

    /// Underscore
    #[token("_")]
    Underscore,

    /// At symbol
    #[token("@")]
    At,

    /// Newline
    #[token("\n")]
    Newline,

    /// Whitespace
    #[regex(r"[ ]+")]
    #[regex(r"\t")]
    WhiteSpace,

    /// Binary value
    ///
    /// Takes form X'bY where X is the bit width and Y is the desired value
    #[regex(r"\d+'b\d+")]
    BinaryValue,

    /// Hi-Z value
    ///
    /// Takes form X'bz where X is the bit width
    #[regex(r"\d+'bz")]
    HiZValue,

    /// Comment start
    #[regex(r"//.*\n")]
    Comment,

    /// Generic text
    #[regex(r"[a-zA-Z]+")]
    Word,

    /// Integer value
    #[regex(r"[0-9]+", |lex| lex.slice().parse())]
    Integer(u8),
}

/// Reads a SystemVerilog file to string for parsing
pub fn read_sv_file(path: &std::path::PathBuf) -> Result<String, std::io::Error> {
    trace!("reading sv file {:?}", path);

    Ok(fs::read_to_string(path)?)
}

/// Simulation object
///
/// Contains file metadata and modules
#[derive(Default)]
pub struct SimObject {
    /// Simulation timing information
    sim_time: SimTime,

    /// Object modules
    mods: Vec<Module>,
}

impl fmt::Debug for SimObject {
    fn fmt(&self, _: &mut std::fmt::Formatter) -> fmt::Result {
        debug!("{:?}", self.sim_time);

        for module in &self.mods {
            format!("{module:?}");
        }
        Ok(())
    }
}

/// Parses a read SystemVerilog file
///
/// At this time, `parse_sv_file` can only return a single error
pub fn parse_sv_file(file_contents: String) -> Result<SimObject, LexingError> {
    let mut lexer = Token::lexer(file_contents.as_str());
    let mut sim_time = SimTime::default();
    let mut mods: Vec<Module> = Vec::new();
    let mut errors: Vec<LexingError> = Vec::new();

    trace!("parsing sv file");

    while let Some(token) = lexer.next() {
        match token {
            Ok(Token::Module) => match parse_module(&mut lexer) {
                Ok(module) => mods.push(module),
                Err(e) => errors.push(e),
            },
            Ok(Token::BTick) => match parse_sim_time(&mut lexer) {
                Ok(val) => sim_time = val,
                Err(e) => errors.push(e),
            },
            Ok(Token::Comment) => match parse_comment(&mut lexer) {
                Ok(_) => (),
                Err(e) => errors.push(e),
            },
            Ok(Token::Newline) | Ok(Token::WhiteSpace) => (),
            Err(e) => {
                error!(
                    "unexpected error occurred parsing sv file: '{}'",
                    lexer.slice()
                );
                return Err(e);
            }
            _ => warn!("{:?} not implemented", token.unwrap()),
        }
    }

    for error in errors {
        error!("lexing error parsing sv file: {:?}", error);
    }

    Ok(SimObject { sim_time, mods })
}

/// SystemVerilog module representation
///
/// Contains a module I/O header, variable assignments, combinational
/// and sequential logic, as well as any constants
#[derive(Default)]
pub struct Module {
    /// Module friendly name
    name: String,

    /// Module I/O information
    io: ModuleIO,

    /// Module "variables" (wire, reg, etc.)
    vars: Vec<Var>,
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

fn parse_module<'source>(lexer: &mut Lexer<'source, Token>) -> Result<Module, LexingError> {
    let mut in_wire = false;
    let mut in_reg = false;
    let mut vars: Vec<Var> = Vec::new();

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
                    Err(_) => error!(
                        "unexpected error occurred parsing module wire: '{}'",
                        lexer.slice()
                    ),
                },
                Err(e) => {
                    error!(
                        "unexpected error occurred parsing module wire: '{}'",
                        lexer.slice()
                    );
                    return Err(e);
                }
                _ => debug!("wire search throwing out {:?}", token),
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
                    Err(_) => error!(
                        "unexpected error occurred parsing module name: '{}'",
                        lexer.slice()
                    ),
                },
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

/// Simulation time command
///
/// Simulation time can be set by using a command such
/// as `timescale 1ns/1ps
#[derive(Debug, Clone, Copy)]
pub struct SimTime {
    /// Numerator time given in ns or ps
    pub n_time: f64,

    /// Denominator time given in ns or ps
    pub d_time: f64,
}

/// Default implementation
///
/// Sets the numerator time to 1ns and the denominator
/// time to 1ps
impl Default for SimTime {
    fn default() -> Self {
        Self {
            n_time: 0.000_001,
            d_time: 0.000_000_001,
        }
    }
}

fn parse_sim_time<'source>(lexer: &mut Lexer<'source, Token>) -> Result<SimTime, LexingError> {
    let mut n_time = 0.;
    let mut d_time = 0.;
    let mut timescale_started = false;
    let mut n_found = false;
    let mut n_search = true;

    trace!("parsing sim time");

    while let Some(token) = lexer.next() {
        match token {
            Ok(Token::Timescale) => {
                if timescale_started {
                    trace!("timescale_started check failed");
                    return Err(LexingError::UnexpectedToken);
                }

                timescale_started = true;
            }
            Ok(Token::Time(val)) => {
                if !timescale_started {
                    trace!("not timescale_started check failed");
                    return Err(LexingError::UnexpectedToken);
                }

                if n_search {
                    if n_found {
                        trace!("n_found check failed");
                        return Err(LexingError::ImproperTimeFormatting);
                    }

                    n_time = val;
                    n_found = true;
                } else {
                    d_time = val;
                    break;
                }
            }
            Ok(Token::Divide) => n_search = false,
            Ok(Token::WhiteSpace) => (),
            Err(e) => {
                error!(
                    "unexpected error occurred parsing sim time: '{}'",
                    lexer.slice()
                );
                return Err(e);
            }
            _ => error!("Unexpected token in timescale {:?}", token.unwrap()),
        }
    }

    Ok(SimTime { n_time, d_time })
}

fn picosecond(lex: &mut Lexer<Token>) -> Option<f64> {
    let slice = lex.slice();
    let n: Result<f64, _> = slice[..slice.len() - 2].parse();

    match n {
        Ok(val) => Some(val * 0.000_000_001),
        Err(e) => {
            error!("could not read picosecond time: {}", e);
            None
        }
    }
}

fn nanosecond(lex: &mut Lexer<Token>) -> Option<f64> {
    let slice = lex.slice();
    let n: Result<f64, _> = slice[..slice.len() - 2].parse();

    match n {
        Ok(val) => Some(val * 0.000_001),
        Err(e) => {
            error!("could not read nanosecond time: {}", e);
            None
        }
    }
}

/// Module I/O information
///
/// Stores all inputs, outputs, and inouts for a given module
#[derive(Default)]
pub struct ModuleIO {
    name: String,
    inputs: Vec<Input>,
    outputs: Vec<Output>,
    inouts: Vec<Inout>,
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

fn parse_comment<'source>(lexer: &mut Lexer<'source, Token>) -> Result<(), LexingError> {
    while let Some(token) = lexer.next() {
        match token {
            Ok(Token::Newline) => return Ok(()),
            Err(e) => {
                error!(
                    "unexpected error occurred parsing module semicolon: '{}'",
                    lexer.slice()
                );
                return Err(e);
            }
            _ => (),
        };
    }
    Ok(())
}
