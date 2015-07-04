use std::fmt::{self, Display, Formatter};

pub use self::ParseError::*;
use com;
use edit;
use vis;

#[derive(Debug, Clone)]
pub enum ParseError {
	GeneralError,
	FloatParseError(vis::VExprRef, usize, usize), // Expr, from, to
	OverflowError,
	SyntaxError,
	StackExhausted(usize),
	UndefVar(char, usize),
	IllegalChar(char, usize),
	IllegalCommand(com::Command, usize),
	IllegalToken(vis::VToken, edit::Cursor),
	UnmatchedParen(usize),
	LastResultNotInitialized,
}

impl Display for ParseError {
	fn fmt(&self, f: &mut Formatter) -> fmt::Result {
		match self {
			&GeneralError      => write!(f, "general error"),
			&FloatParseError(ref ex, ref from, ref to) => {
				let mut s = String::new();
				try!(vis::display_vexpr(ex.clone(), &None, &mut s));
				write!(f, "float parsing error from {} to {} in expression `{}`", from, to, s)
			},
			&OverflowError     => write!(f, "integer overflow"),
			&SyntaxError       => write!(f, "syntax error"),
			&StackExhausted(ref pos)  => write!(f, "stack exhausted at {}", pos),
			&UndefVar(ref c, ref pos) => write!(f, "undefined variable referenced ({} at {})", c, pos),
			&IllegalChar(ref c, ref pos) => write!(f, "illegal character ({:?} at {})", c, pos),
			&IllegalCommand(ref c, ref pos) => write!(f, "illegal command ({:?} at {})", c, pos),
			&IllegalToken(ref tok, ref cursor) => {
				let mut s = String::new();
				try!(vis::display_vexpr(cursor.ex.clone(), &None, &mut s));
				write!(f, "illegal token ({:?} at {} in expression `{}`", tok, cursor.pos, s)
			},
			&UnmatchedParen(ref pos)  => write!(f, "unmatched parenthesis encountered at {}", pos),
			&LastResultNotInitialized => write!(f, "last result not initialized"),
		}
	}
}
