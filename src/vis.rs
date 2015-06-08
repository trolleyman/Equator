use	std::rc::Rc;
use std::rc::Weak;
use std::cell::RefCell;

use self::VToken::*;
use self::ParseErrorType::*;

use func::FuncType;

#[derive(Debug)]
pub struct ParseError {
	err: ParseErrorType,
	pos: usize
}
impl ParseError {
	pub fn new(err: ParseErrorType, pos: usize) -> ParseError {
		ParseError{err:err, pos:pos}
	}
	pub fn at_pos(pos: usize) -> ParseError {
		ParseError::new(GeneralError, pos)
	}
}

#[derive(Clone, Debug)]
pub enum ParseErrorType {
	GeneralError,
	OverflowError
}

#[derive(Clone, Debug)]
pub enum VToken {
	Char(char),
	Exp(Rc<RefCell<VExpr>>),
	//Root(Rc<RefCell<VExpr>>, Rc<RefCell<VExpr>>),
	Func(FuncType, Rc<RefCell<VExpr>>)
}
impl VToken {
	/// Gets the inner expression held within the token.
	/// Returns None if the token doesn't contain an inner function.
	pub fn get_inner_expr(&self) -> Option<Rc<RefCell<VExpr>>> {
		match self {
			&Exp(ref ex) | &Func(_, ref ex) => Some(ex.clone()),
			&Char(_) => None
		}
	}
	
	pub fn has_inner_expr(&self) -> bool {
		match self {
			&Exp(_) | &Func(_, _) => true,
			&Char(_) => false
		}
	}
}

pub type VExprRef = Rc<RefCell<VExpr>>;

#[derive(Debug)]
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
}
impl Clone for VExpr {
	fn clone(&self) -> Self {
		VExpr{tokens: self.tokens.clone(), parent: self.parent.clone()}
	}
}