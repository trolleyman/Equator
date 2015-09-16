use std::fmt::{self, Display, Formatter};

pub use self::ParseError::*;
use com;
use edit;
use vis;

#[derive(Debug, Clone)]
pub enum ParseError {
	GeneralError,
	NumParseError(vis::VExprRef, usize, usize), // Expr, from, to
	SyntaxError,
	CommandExecuteError(com::Command, usize),
	StackExhausted(usize),
	UndefVar(char, usize),
	IllegalChar(char, usize),
	IllegalCommand(com::Command, usize),
	IllegalToken(vis::VToken, edit::Cursor),
	UnmatchedParen(usize),
	ExpressionEmpty,
	NoLastResult,
}

impl Display for ParseError {
	fn fmt(&self, f: &mut Formatter) -> fmt::Result {
		match self {
			&GeneralError              => write!(f, "general error"),
			&NumParseError(_, _, _)    => write!(f, "number parsing error"),
			&SyntaxError               => write!(f, "syntax error"),
			&CommandExecuteError(_, _) => write!(f, "command execution error"),
			&StackExhausted(_)         => write!(f, "stack exhausted"),
			&UndefVar(ref c, _)        => write!(f, "undefined variable referenced '{}'", c),
			&IllegalChar(ref c, _)     => write!(f, "illegal character '{}'", c),
			&IllegalCommand(ref c, _)  => write!(f, "illegal command '{:?}'", c),
			&IllegalToken(ref tok, _)  => write!(f, "illegal token '{:?}'", tok),
			&UnmatchedParen(_)         => write!(f, "unmatched parenthesis encountered"),
			&ExpressionEmpty           => write!(f, "expression empty"),
			&NoLastResult              => write!(f, "no last result calculated"),
		}
	}
}
