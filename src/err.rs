use std::fmt::{self, Display, Formatter};

pub use self::ParseError::*;

pub enum ParseError {
	GeneralError,
	OverflowError,
	SyntaxError,
	StackExhausted,
	IllegalChar,
	UnmatchedParen,
}

impl Display for ParseError {
	fn fmt(&self, f: &mut Formatter) -> Result<(), fmt::Error> {
		match self {
			&GeneralError   => f.write_str("general error"),
			&OverflowError  => f.write_str("integer overflow"),
			&SyntaxError    => f.write_str("syntax error"),
			&StackExhausted => f.write_str("stack exhausted"),
			&IllegalChar    => f.write_str("illegal character"),
			&UnmatchedParen => f.write_str("unmatched parenthesis encountered"),
		}
	}
}
