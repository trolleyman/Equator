use	std::rc::Rc;
use std::rc::Weak;
use std::cell::RefCell;
use std::fmt::{self, Display, Formatter};
use std::fmt::Write;

use self::VToken::*;

use err::*;
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
pub struct Cursor {
	pub ex: VExprRef,
	pub pos: usize,
}

impl Cursor {
	pub fn new() -> Cursor {
		Cursor{ex:VExpr::new_ref(), pos:0}
	}
	pub fn new_ex(ex:VExprRef, pos:usize) -> Cursor {
		Cursor{ex:ex, pos:pos}
	}
	pub fn with_ex(ex:VExprRef) -> Cursor {
		Cursor{ex:ex, pos:0}
	}
	
	/// Performs a delete at `self.pos`. If successful, return true.
	pub fn delete(&mut self) -> bool {
		let pos_in_bounds = {
			self.ex.borrow().tokens.get(self.pos).is_some()
		};
		// If token at pos, delete that.
		if pos_in_bounds {
			let ex_clone = self.ex.clone();
			let mut ex = ex_clone.borrow_mut();
			ex.tokens.remove(self.pos);
		} else {
			// Else, try and delete parent.
			if self.move_out() {
				let ex_clone = self.ex.clone();
				let mut ex = ex_clone.borrow_mut();
				ex.tokens.remove(self.pos);
			} else {
				return false;
			}
		}
		return true;
	}
	
	/// Performs a backsace at `self.pos`. If successful, returns true.
	pub fn backspace(&mut self) -> bool {
		if self.move_left() {
			return self.delete();
		} else {
			return false;
		}
	}
	
	/// Moves the cursor left one position. If successful, return true.
	pub fn move_left(&mut self) -> bool {
		if self.pos == 0 || self.ex.borrow().tokens.get(self.pos - 1).is_none() {
			// Move up to the parent, if there is one
			let orig_ex = self.ex.clone();
			if !self.move_out() {
				return false;
			}
			let exprs = self.ex.borrow().tokens[self.pos].get_inner_expr();
			let mut found: isize = -1;
			for i in 0..exprs.len() {
				if unsafe { exprs[i].as_unsafe_cell().get() == orig_ex.as_unsafe_cell().get() } {
					found = i as isize;
					break;
				}
			}
			if found <= 0 || found as usize >= exprs.len() {
				//if self.pos != 0 {
				//	//self.pos -= 1;
				//} else {
				//	return false;
				//}
			} else {
				self.ex = exprs[found as usize - 1].clone();
				self.pos = self.ex.borrow().tokens.len();
			}
			return true;
		} else {
			// Try and drill down into a token
			self.pos -= 1;
			let exprs = self.ex.borrow().tokens[self.pos].get_inner_expr();
			if exprs.len() != 0 {
				self.ex = exprs[exprs.len() - 1].clone();
				self.pos = self.ex.borrow().tokens.len();
			}
			return true;
		}
	}
	
	/// Moves the cursor right one position. If successful, return true.
	///
	/// Example progression:
	/// 2|3^(98)+ => 23|^(98)+ => 23|^(98)+ => 23^(|98)+ => 23^(9|8)+ => 23^(98|)+ => 23^(98)|+ => 23^(98)+| => 23^(98)+| ... etc.
	pub fn move_right(&mut self) -> bool {
		let orig_pos = self.pos;
		let orig_ex = self.ex.clone();
		
		// Try move down
		if !self.move_in() {
			// If not possible, move forward.
			self.pos += 1;
			if self.pos > self.ex.borrow().tokens.len() {
				// If out of range, move up + forward.
				if !self.move_out() {
					// If not successful, move back again to original place.
					self.pos = orig_pos;
					return false;
				} else {
					let exprs = self.ex.borrow().tokens[self.pos].get_inner_expr();
					let mut found: isize = -1;
					for i in 0..exprs.len() {
						if unsafe { exprs[i].as_unsafe_cell().get() == orig_ex.as_unsafe_cell().get() } {
							found = i as isize;
							break;
						}
					}
					if found == -1 || found as usize >= exprs.len() - 1 {
						if self.pos < self.ex.borrow().tokens.len() {
							self.pos += 1;
						} else {
							return false;
						}
					} else {
						self.ex = exprs[found as usize+1].clone();
						self.pos = 0;
					}
				}
			}
		}
		return true;
	}
	
	/// Trys to move the cursor down into the current token. Returns true if the operation was successful.
	pub fn move_in(&mut self) -> bool {
		let ex_clone = self.ex.clone();
		let ex = ex_clone.borrow();
		let tok = match ex.tokens.get(self.pos).clone() {
			Some(t) => t,
			None => return false
		};
		match tok.get_inner_expr().get(0) {
			Some(expr) => {
				self.ex = expr.clone();
				self.pos = 0;
				true
			},
			None => false
		}
	}
	
	/// Moves the cursor to the parent token of the current token. Returns true if the operation was successful.
	pub fn move_out(&mut self) -> bool {
		let ex_clone = self.ex.clone();
		let ex = ex_clone.borrow();
		if ex.parent.is_some() {
			let parent_weak = ex.clone().parent.unwrap();
			if let Some(parent) = parent_weak.upgrade() {
				
				self.ex = parent;
				// Right expr, wrong place. Find original ex in parent.
				// Panic if not found, this signals some terrible breakdown in the structure of the expression.
				let mut found = false;
				let tokens = self.ex.borrow().clone().tokens;
				for i in 0..tokens.len() {
					unsafe {
						let tok = tokens[i].clone();
						for inner_ex in tok.get_inner_expr().iter() {
							if inner_ex.as_unsafe_cell().get() == ex_clone.as_unsafe_cell().get() {
								found = true;
								self.pos = i;
								break;
							}
						}
					}
				}
				if !found {
					panic!("token could not be found in parent expression");
				}
				return true;
			}
		}
		return false;
	}
	
	/// Move visually up.
	pub fn move_up(&mut self) -> bool {
		// If in a token with multiple inner expressions, move to the left of the current one.
		let parent_ex = match self.ex.borrow().get_parent() {
			Some(ex) => ex,
			None => return false,
		};
		let (i, j) = match find_vexpr(&self.ex, &parent_ex) {
			Some((i, j)) => (i, j),
			None         => return false,
		};
		let current_token = parent_ex.borrow().tokens[i].clone();
		let exprs = current_token.get_inner_expr();
		if j == 0 {
			false
		} else {
			self.ex  = exprs[j - 1].clone();
			self.pos = self.ex.borrow().tokens.len();
			true
		}
	}
	
	pub fn move_down(&mut self) -> bool {
		// If in a token with multiple inner expressions, move to the right of the current one.
		let parent_ex = match self.ex.borrow().get_parent() {
			Some(ex) => ex,
			None => return false,
		};
		let (i, j) = match find_vexpr(&self.ex, &parent_ex) {
			Some((i, j)) => (i, j),
			None         => return false,
		};
		let current_token = parent_ex.borrow().tokens[i].clone();
		let exprs = current_token.get_inner_expr();
		if j >= exprs.len() - 1 {
			false
		} else {
			self.ex  = exprs[j + 1].clone();
			self.pos = 0;
			true
		}
	}
	
	pub fn is_in_spans(&self, spans: &[Span]) -> bool {
		for span in spans.iter() {
			if span.contains(self) {
				return true;
			}
		}
		false
	}
	
	pub fn is_in_errors(&self, errors: &[VError]) -> bool {
		for error in errors.iter() {
			if error.span.contains(self) {
				return true;
			}
		}
		false
	}
}

#[derive(Clone, Debug)]
pub struct Span {
	pub ex: VExprRef,
	pub start: usize,
	pub end: usize,
}

impl Span {
	pub fn new(ex: VExprRef, start: usize, end: usize) -> Span {
		Span{ ex:ex, start:start, end:end }
	}
	pub fn contains(&self, cur: &Cursor) -> bool {
		if is_equal_reference(&self.ex, &cur.ex) {
			if self.start == self.end {
				cur.pos >= self.start && cur.pos <= self.end
			} else {
				cur.pos >= self.start && cur.pos < self.end
			}
		} else {
			false
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
	pub fn from_char(c: char) -> VToken {
		if c.is_digit(10) {
			VToken::Digit(c)
		} else {
			match c {
				'+'            => VToken::Op(OpType::Add),
				'-' | CHAR_SUB => VToken::Op(OpType::Sub),
				'*' | CHAR_MUL => VToken::Op(OpType::Mul),
				'/' | CHAR_DIV => VToken::Op(OpType::Div),
				_ => VToken::Char(c)
			}
		}
	}
	
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

pub fn display_vexpr<T: Write>(ex: VExprRef, cursor_opt: &Option<Cursor>, buf: &mut T) -> fmt::Result {
	let cursor = match cursor_opt {
		&Some(ref c) => c.clone(),
		&None        => Cursor::new()
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
pub fn display_vexpr_errors<T: Write, V: Write>(ex: VExprRef, cursor_opt: &Option<Cursor>, errors: &[VError], buf: &mut T, e_buf: &mut V) -> fmt::Result {
	let cursor = match cursor_opt {
		&Some(ref c) => c.clone(),
		&None        => Cursor::new()
	};
	let cursor_in_ex: bool = is_equal_reference(&ex, &cursor.ex);
	
	let len = ex.borrow().tokens.len();
	for i in 0..len {
		let err = Cursor::new_ex(ex.clone(), i).is_in_errors(errors);
		if cursor_in_ex && cursor.pos == i {
			// Print cursor
			try!(write!(buf, "|"));
			try!(write!(e_buf, " "));
		}
		
		match ex.borrow().tokens[i].clone() {
			VToken::Space => {
				try!(write!(buf, "{}", CHAR_BOX));
				if err { try!(write!(e_buf, "~"));
				} else { try!(write!(e_buf, " ")); }
			},
			VToken::Digit(c) | VToken::Char(c) => {
				try!(write!(buf, "{}", c));
				if err { try!(write!(e_buf, "~"));
				} else { try!(write!(e_buf, " ")); }
			},
			VToken::Op(op) => {
				try!(write!(buf, "{}", op));
				if err { try!(write!(e_buf, "~"));
				} else { try!(write!(e_buf, " ")); }
			},
			VToken::Pow(inner_ex_ref) => {
				// Recursive stuff yay!
				try!(write!(buf, "^("));
				if err { try!(write!(e_buf, "~~"));
				} else { try!(write!(e_buf, "  ")); }
				try!(display_vexpr(inner_ex_ref, &Some(cursor.clone()), buf));
				try!(write!(buf, ")"));
				if err { try!(write!(e_buf, "~"));
				} else { try!(write!(e_buf, " ")); }
			},
			VToken::Func(func_type, inner_ex_ref) => {
				let s = format!("{}", func_type);
				try!(write!(buf, " {}(", s));
				for _ in 0..s.len() + 2 {
					if err { try!(write!(e_buf, "~"));
					} else { try!(write!(e_buf, " ")); }
				}
				try!(display_vexpr(inner_ex_ref, &Some(cursor.clone()), buf));
				try!(write!(buf, ")"));
				if err { try!(write!(e_buf, "~"));
				} else { try!(write!(e_buf, " ")); }
			}
			VToken::Root(degree_ex, inner_ex) => {
				try!(write!(buf, " root("));
				if err { try!(write!(e_buf, "~~~~~~"));
				} else { try!(write!(e_buf, "      ")); }
				try!(display_vexpr(degree_ex, &Some(cursor.clone()), buf));
				try!(write!(buf, ", "));
				if err { try!(write!(e_buf, "~~"));
				} else { try!(write!(e_buf, "  ")); }
				try!(display_vexpr(inner_ex, &Some(cursor.clone()), buf));
				try!(write!(buf, ")"));
				if err { try!(write!(e_buf, "~"));
				} else { try!(write!(e_buf, " ")); }
			}
			VToken::Frac(num_ex, den_ex) => {
				try!(write!(buf, "(("));
				if err { try!(write!(e_buf, "~~"));
				} else { try!(write!(e_buf, "  ")); }
				try!(display_vexpr(num_ex, &Some(cursor.clone()), buf));
				try!(write!(buf, ")÷("));
				if err { try!(write!(e_buf, "~~~"));
				} else { try!(write!(e_buf, "   ")); }
				try!(display_vexpr(den_ex, &Some(cursor.clone()), buf));
				try!(write!(buf, "))"));
				if err { try!(write!(e_buf, "~~"));
				} else { try!(write!(e_buf, "  ")); }
			}
		}
	}

	let err = Cursor::new_ex(ex.clone(), 0).is_in_errors(errors);
	if cursor_in_ex && cursor.pos == ex.borrow().tokens.len() {
		// Print cursor
		if len == 0 {
			try!(write!(buf, "{}", CHAR_HLBOX));
			if err { try!(write!(e_buf, "~"));
			} else { try!(write!(e_buf, " ")); }
		} else {
			try!(write!(buf, "|"));
			try!(write!(e_buf, " "));
		}
	} else if len == 0 {
		try!(write!(buf, "{}", CHAR_BOX));
		if err { try!(write!(e_buf, "~"));
		} else { try!(write!(e_buf, " ")); }
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
