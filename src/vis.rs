use	std::rc::Rc;
use std::rc::Weak;
use std::cell::RefCell;

use self::VToken::*;

use func::FuncType;

#[derive(Clone, Debug)]
pub enum VToken {
	Char(char),
	Pow(VExprRef),
	Root(VExprRef, VExprRef),
	Func(FuncType, VExprRef)
}
impl VToken {
	pub fn get_inner_expr(&self) -> Box<[VExprRef]> {
		match self {
			&Pow(ref ex) | &Func(_, ref ex) => box [ex.clone()],
			&Root(ref ex1, ref ex2) => box [ex1.clone(), ex2.clone()],
			&Char(_) => box []
		}
	}
	
	pub fn has_inner_expr(&self) -> bool {
		match self {
			&Pow(_) | &Func(_, _) | &Root(_, _) => true,
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