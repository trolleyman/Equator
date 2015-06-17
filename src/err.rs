use std::fmt::{self, Display, Formatter};

pub use self::ParseError::*;
use com;

#[derive(Debug, Clone, Copy)]
pub enum ParseError {
	GeneralError,
	FloatParseError,
	OverflowError,
	SyntaxError,
	StackExhausted,
	IllegalChar(char, usize),
	IllegalCommand(com::Command, usize),
	UnmatchedParen,
	LastResultNotInitialized,
}

impl Display for ParseError {
	fn fmt(&self, f: &mut Formatter) -> Result<(), fmt::Error> {
		match self {
			&GeneralError      => write!(f, "general error"),
			&FloatParseError   => write!(f, "float parsing error"),
			&OverflowError     => write!(f, "integer overflow"),
			&SyntaxError       => write!(f, "syntax error"),
			&StackExhausted    => write!(f, "stack exhausted"),
			&IllegalChar(ref c, ref pos) => write!(f, "illegal character ({:?} at {})", c, pos),
			&IllegalCommand(ref c, ref pos) => write!(f, "illegal command ({:?} at {})", c, pos),
			&UnmatchedParen    => write!(f, "unmatched parenthesis encountered"),
			&LastResultNotInitialized => write!(f, "last result not initialized"),
		}
	}
}
