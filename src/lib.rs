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

/// Module type definitions
pub mod module;
use module::*;

/// Logic parsing and simulation engine
pub mod logic;
pub use logic::*;

/// Testbench utilities for simulation
pub mod testbench;
pub use testbench::*;

/// Enhanced parser for SystemVerilog
pub mod parser;
pub use parser::*;

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

    /// Always block
    #[token("always")]
    Always,

    /// Always_ff (sequential)
    #[token("always_ff")]
    AlwaysFF,

    /// Case statement
    #[token("case")]
    Case,

    /// Casez statement (with don't care)
    #[token("casez")]
    Casez,

    /// Casex statement (with don't care)
    #[token("casex")]
    Casex,

    /// Default case
    #[token("default")]
    Default,

    /// Endcase
    #[token("endcase")]
    Endcase,

    /// For loop
    #[token("for")]
    For,

    /// While loop
    #[token("while")]
    While,

    /// Generate block
    #[token("generate")]
    Generate,

    /// Endgenerate
    #[token("endgenerate")]
    Endgenerate,

    /// Genvar
    #[token("genvar")]
    Genvar,

    /// Function
    #[token("function")]
    Function,

    /// Endfunction
    #[token("endfunction")]
    Endfunction,

    /// Task
    #[token("task")]
    Task,

    /// Endtask
    #[token("endtask")]
    Endtask,

    /// Return
    #[token("return")]
    Return,

    /// Localparam
    #[token("localparam")]
    Localparam,

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

    /// Hex value
    ///
    /// Takes form `X'hY` where `X` is the bit width and `Y` is the hex value
    #[regex(r"\d+'h[0-9a-fA-F]+")]
    HexValue,

    /// Bitwise NOT
    #[token("~")]
    Tilde,

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
            let _ = format!("{module:?}");
        }
        Ok(())
    }
}

/// Parses a read SystemVerilog file using the enhanced parser
pub fn parse_sv_file(file_contents: String) -> Result<SimObject, LexingError> {
    trace!("parsing sv file with enhanced parser");

    let mut parser = SVParser::new(&file_contents);
    let mut sim_time = SimTime::default();
    let mut mods: Vec<Module> = Vec::new();

    // Parse the entire file
    loop {
        parser.skip_whitespace();

        match parser.current() {
            Some(Ok(Token::BTick)) => {
                // Parse timescale - for now just skip it
                parser.advance();
                while let Some(Ok(token)) = parser.current() {
                    if matches!(token, Token::Newline) {
                        parser.advance();
                        break;
                    }
                    parser.advance();
                }
            }
            Some(Ok(Token::Module)) => {
                let module = parser.parse_module()?;
                mods.push(module);
            }
            Some(Ok(Token::Comment)) => {
                parser.skip_comment();
            }
            Some(Ok(_)) => {
                // Skip unknown tokens
                trace!("Skipping unknown token: {:?}", parser.current());
                parser.advance();
            }
            Some(Err(e)) => {
                error!("Lexer error: {:?}", e);
                return Err(e.clone());
            }
            None => break,
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
