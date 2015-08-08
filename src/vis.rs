use	std::rc::Rc;
use std::rc::Weak;
use std::cell::RefCell;
use std::fmt::{self, Display, Formatter};
use std::fmt::Write;

use self::VToken::*;

use edit;
use consts::*;
use func::FuncType;

#[derive(Clone, Debug)]
pub enum OpType {
	Add,
	Sub,
	Mul,
	Div,
}
impl Display for OpType {
	fn fmt(&self, f: &mut Formatter) -> fmt::Result {
		match self {
			&OpType::Add => write!(f, "{}", CHAR_ADD),
			&OpType::Sub => write!(f, "{}", CHAR_SUB),
			&OpType::Mul => write!(f, "{}", CHAR_MUL),
			&OpType::Div => write!(f, "{}", CHAR_DIV),
		}
	}
}

#[derive(Clone, Debug)]
pub enum VToken {
	Space,
	Char(char),
	Digit(char),
	Op(OpType),
	Pow(VExprRef),
	Frac(VExprRef, VExprRef), // (numerator, denominator)
	Root(VExprRef, VExprRef),
	Func(FuncType, VExprRef),
}
impl VToken {
	pub fn get_inner_expr(&self) -> Box<[VExprRef]> {
		match self {
			&Pow(ref ex) | &Func(_, ref ex) => box [ex.clone()],
			&Root(ref ex1, ref ex2) | &Frac(ref ex1, ref ex2) => box [ex1.clone(), ex2.clone()],
			&Op(_) | &Digit(_) | &Char(_) | &Space => box []
		}
	}
	
	pub fn has_inner_expr(&self) -> bool {
		match self {
			&Pow(_) | &Func(_, _) | &Root(_, _) | &Frac(_, _) => true,
			&Op(_) | &Digit(_) | &Char(_) | &Space => false
		}
	}
}

pub type VExprRef = Rc<RefCell<VExpr>>;

#[derive(Clone, Debug)]
pub struct VExpr {
	pub tokens: Vec<VToken>,
	pub parent: Option<Weak<RefCell<VExpr>>>,
}
impl VExpr {
	pub fn new() -> VExpr {
		VExpr{tokens: Vec::new(), parent: None}
	}
	pub fn with_parent(ex: Rc<RefCell<VExpr>>) -> VExpr {
		VExpr{tokens: Vec::new(), parent: Some(ex.downgrade())}
	}
	
	pub fn new_ref() -> Rc<RefCell<VExpr>> {
		Rc::new(RefCell::new(VExpr::new()))
	}
	pub fn to_ref(&self) -> Rc<RefCell<VExpr>>  {
		Rc::new(RefCell::new((*self).clone()))
	}
	pub fn get_parent(&self) -> Option<VExprRef> {
		match self.parent {
			Some(ref weak) => weak.upgrade(),
			None => None
		}
	}
}
impl Display for VExpr {
	fn fmt(&self, f: &mut Formatter) -> fmt::Result {
		display_vexpr(self.to_ref(), &None, f)
	}
}

pub fn display_vexpr<T: Write>(ex: VExprRef, cursor_opt: &Option<edit::Cursor>, buf: &mut T) -> fmt::Result {
	let cursor = match cursor_opt {
		&Some(ref c) => c.clone(),
		&None        => edit::Cursor::new()
	};
	let cursor_in_ex: bool = is_equal_reference(&ex, &cursor.ex);
	
	let len = ex.borrow().tokens.len();
	for i in 0..len {
		if cursor_in_ex && cursor.pos == i {
			// Print cursor
			try!(write!(buf, "|"));
		}
		
		match ex.borrow().tokens[i].clone() {
			VToken::Space => {
				try!(write!(buf, "{}", CHAR_BOX));
			},
			VToken::Digit(c) | VToken::Char(c) => {
				try!(write!(buf, "{}", c));
			},
			VToken::Op(op) => {
				try!(write!(buf, "{}", op));
			},
			VToken::Pow(inner_ex_ref) => {
				// Recursive stuff yay!
				try!(write!(buf, "^("));
				try!(display_vexpr(inner_ex_ref, &Some(cursor.clone()), buf));
				try!(write!(buf, ")"));
			},
			VToken::Func(func_type, inner_ex_ref) => {
				try!(write!(buf, " {}(", func_type));
				try!(display_vexpr(inner_ex_ref, &Some(cursor.clone()), buf));
				try!(write!(buf, ")"));
			}
			VToken::Root(degree_ex, inner_ex) => {
				try!(write!(buf, " root("));
				try!(display_vexpr(degree_ex, &Some(cursor.clone()), buf));
				try!(write!(buf, ", "));
				try!(display_vexpr(inner_ex, &Some(cursor.clone()), buf));
				try!(write!(buf, ")"));
			}
			VToken::Frac(num_ex, den_ex) => {
				try!(write!(buf, "(("));
				try!(display_vexpr(num_ex, &Some(cursor.clone()), buf));
				try!(write!(buf, ")÷("));
				try!(display_vexpr(den_ex, &Some(cursor.clone()), buf));
				try!(write!(buf, "))"));
			}
		}
	}

	if cursor_in_ex && cursor.pos == ex.borrow().tokens.len() {
		// Print cursor
		if len == 0 {
			try!(write!(buf, "{}", CHAR_HLBOX));
		} else {
			try!(write!(buf, "|"));
		}
	} else if len == 0 {
		try!(write!(buf, "{}", CHAR_BOX));
	}
	Ok(())
}

pub fn is_equal_reference<T>(ref1: &Rc<RefCell<T>>, ref2: &Rc<RefCell<T>>) -> bool {
	unsafe { ref1.as_unsafe_cell().get() == ref2.as_unsafe_cell().get() }
}

/// Tries to find `needle` in `hayastack`.
/// Gives the token position and the position in that token.
pub fn find_vexpr(needle: &VExprRef, haystack: &VExprRef) -> Option<(usize, usize)> {
	let mut i = 0;
	for tok in haystack.borrow().tokens.iter() {
		let mut j = 0;
		for ex_inner in tok.get_inner_expr().iter() {
			if is_equal_reference(&ex_inner, &needle) {
				return Some((i, j));
			}
			j += 1;
		}
		i += 1;
	}
	None
}
