use std::fmt::{self, Display, Formatter};

pub use self::ErrorType::*;
use com;
use vis;

#[derive(Clone)]
pub struct VError {
	pub error: ErrorType,
	pub span: vis::Span,
}
impl VError {
	pub fn new(error: ErrorType, ex: VExprRef, start: usize, end: usize) -> VError {
		VError::from_span(error, vis::Span::new(ex, start, end))
	}
	pub fn from_span(error: ErrorType, span: vis::Span) -> VError {
		VError{ error: error, span: span }
	}
}
impl Display for VError {
	fn fmt(&self, f: &mut Formatter) -> fmt::Result {
		write!(f, "{}", self.error)
	}
}

#[derive(Debug, Clone)]
pub enum ErrorType {
	// Compile errors
	GeneralError,
	NumParseError,
	CommandExecuteError,
	StackExhausted,
	UndefVar(char),
	IllegalChar(char),
	IllegalCommand(com::Command),
	IllegalToken(vis::VToken),
	UnmatchedParen,
	ExpressionEmpty,
	// Runtime errors
	
}

impl Display for ErrorType {
	fn fmt(&self, f: &mut Formatter) -> fmt::Result {
		match self {
			&GeneralError          => write!(f, "general error"),
			&NumParseError         => write!(f, "number parsing error"),
			&SyntaxError           => write!(f, "syntax error"),
			&CommandExecuteError   => write!(f, "command execution error"),
			&StackExhausted        => write!(f, "stack exhausted"),
			&UndefVar(ref c)       => write!(f, "undefined variable referenced '{}'", c),
			&IllegalChar(ref c)    => write!(f, "illegal character '{}'", c),
			&IllegalCommand(ref c) => write!(f, "illegal command '{:?}'", c),
			&IllegalToken(ref tok) => write!(f, "illegal token '{:?}'", tok),
			&UnmatchedParen        => write!(f, "unmatched parenthesis encountered"),
			&ExpressionEmpty       => write!(f, "expression empty"),
		}
	}
}
