use crate::{LexingError, Token};
use log::{error, trace};
use logos::Lexer;

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

/// Parses simulation timing constraints to completion
pub fn parse_sim_time<'source>(lexer: &mut Lexer<'source, Token>) -> Result<SimTime, LexingError> {
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

/// Parses a time given in picoseconds
pub fn picosecond(lex: &mut Lexer<Token>) -> Option<f64> {
    let slice = lex.slice();
    let n: Result<f64, _> = slice[..slice.len() - 2].parse();

    trace!("parsing picosecond");

    match n {
        Ok(val) => Some(val * 0.000_000_001),
        Err(e) => {
            error!("could not read picosecond time: {}", e);
            None
        }
    }
}

/// Parses a time given in nanoseconds
pub fn nanosecond(lex: &mut Lexer<Token>) -> Option<f64> {
    let slice = lex.slice();
    let n: Result<f64, _> = slice[..slice.len() - 2].parse();

    trace!("parsing nanosecond");

    match n {
        Ok(val) => Some(val * 0.000_001),
        Err(e) => {
            error!("could not read nanosecond time: {}", e);
            None
        }
    }
}
