use std::fmt::{self, Display, Formatter};

pub use self::ParseError::*;
use com;
use vis;

#[derive(Debug, Clone)]
pub enum ParseError {
	GeneralError,
	FloatParseError,
	OverflowError,
	SyntaxError,
	StackExhausted,
	IllegalChar(char, usize),
	IllegalCommand(com::Command, usize),
	IllegalToken(vis::VToken, vis::VExprRef, usize),
	UnmatchedParen,
	LastResultNotInitialized,
}

impl Display for ParseError {
	fn fmt(&self, f: &mut Formatter) -> fmt::Result {
		match self {
			&GeneralError      => write!(f, "general error"),
			&FloatParseError   => write!(f, "float parsing error"),
			&OverflowError     => write!(f, "integer overflow"),
			&SyntaxError       => write!(f, "syntax error"),
			&StackExhausted    => write!(f, "stack exhausted"),
			&IllegalChar(ref c, ref pos) => write!(f, "illegal character ({:?} at {})", c, pos),
			&IllegalCommand(ref c, ref pos) => write!(f, "illegal command ({:?} at {})", c, pos),
			&IllegalToken(ref tok, ref ex, ref pos) => {
				let mut s = String::new();
				try!(vis::display_vexpr(ex.clone(), &None, &mut s));
				write!(f, "illegal token ({:?} at {} in expression `{}`", tok, pos, s)
			},
			&UnmatchedParen    => write!(f, "unmatched parenthesis encountered"),
			&LastResultNotInitialized => write!(f, "last result not initialized"),
		}
	}
}
