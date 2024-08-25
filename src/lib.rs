#![doc = include_str!("../README.md")]
#![doc(html_logo_url = "https://github.com/DMoore12/sv-sim/blob/main/sv-sim-logo.png?raw=true")]

/// Logging
use log::{debug, error, trace, warn};
use logos::{Lexer, Logos};
use std::num::ParseIntError;

/// File reading/writing
use std::fs;

/// Debug
use std::fmt;

/// Variable types and parsing
pub mod var_types;
// use var_types::*;

/// Simulation timing constraints and parsing
pub mod sim_time;
use sim_time::*;

/// Module type and parsing
pub mod module;
use module::*;

/// Errors occurring due to incorrect character sequences
#[derive(Default, Debug, Clone, PartialEq)]
pub enum LexingError {
    /// Invalid integer found
    InvalidInteger(String),

    /// Unexpected token or generic error
    #[default]
    UnexpectedToken,

    /// Unexpected token, expected semicolon
    ExpectedSemi,

    /// Improper time format found
    ImproperTimeFormatting,

    /// Improper comment format found
    ImproperCommentFormatting,

    /// Non ASCII character found (not currently used)
    NonAsciiCharacter,

    /// Bit width opened but not closed
    IncompleteWidth,

    /// Bit width determined to be negative
    NegativeBitWidth,

    /// Module wire parsing failed
    ModuleWireNotFound,
}

impl Into<String> for LexingError {
    fn into(self) -> String {
        match self {
            Self::InvalidInteger(error) => format!("invalid integer encountered: {error:}"),
            Self::UnexpectedToken => "unexpected token encountered".to_owned(),
            Self::ImproperTimeFormatting => "improper time format encountered".to_owned(),
            Self::IncompleteWidth => "incomplete width encountered".to_owned(),
            Self::NegativeBitWidth => "negative bit width encountered".to_owned(),
            _ => "generic/unknown error encountered".to_owned(),
        }
    }
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

/// Lexer token output
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
    /// Takes form `X'bY` where `X` is the bit width and `Y` is the desired value
    #[regex(r"\d+'b\d+")]
    BinaryValue,

    /// Hi-Z value
    ///
    /// Takes form `X'bz` where `X` is the bit width
    #[regex(r"\d+'bz")]
    HiZValue,

    /// Comment start
    #[regex(r"//")]
    Comment,

    /// Generic text
    #[regex(r"[a-zA-Z]+")]
    Word,

    /// Integer value
    #[regex(r"[0-9]+", |lex| lex.slice().parse())]
    Integer(u64),
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
    pub sim_time: SimTime,

    /// Object modules
    pub mods: Vec<Module>,
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

    trace!("parsing sv file");

    while let Some(token) = lexer.next() {
        match token {
            Ok(Token::Module) => mods.push(parse_module(&mut lexer)?),
            Ok(Token::BTick) => sim_time = parse_sim_time(&mut lexer)?,
            Ok(Token::Comment) => parse_comment(&mut lexer)?,
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

    Ok(SimObject { sim_time, mods })
}

fn parse_comment<'source>(lexer: &mut Lexer<'source, Token>) -> Result<(), LexingError> {
    trace!("parsing comment");

    while let Some(token) = lexer.next() {
        match token {
            Ok(Token::Newline) => return Ok(()),
            Err(_) => {
                error!(
                    "unexpected error occurred parsing comment: '{}'",
                    lexer.slice()
                );
                return Err(LexingError::ImproperCommentFormatting);
            }
            _ => (),
        };
    }
    Ok(())
}
