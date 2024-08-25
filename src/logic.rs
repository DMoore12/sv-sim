use crate::var_types::{self, *};
use crate::{parse_comment, LexingError, Token};
use log::{debug, error, trace};
use logos::Lexer;
use std::fmt;

pub fn parse_if_statement<'source>(lexer: &mut Lexer<'source, Token>) -> Result<Logic, LexingError> {

}
