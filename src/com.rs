/// Module for turning the equation into commands
use std::collections::HashMap;
use std::fmt::Write;
use consts::*;
use vis::*;
use func::*;
use err::*;

#[allow(non_snake_case)]
mod Com {
	pub use super::Command::*;
}
#[derive(Debug, Clone, PartialEq)]
pub enum Command {
	Var(char), // Pushes variable with char identifier to the stack
	Int(i64), // Pushes integer literal to the stack
	Float(f64), // Pushed float literal to the stack
	Add, // A, B => A + B
	Sub, // A, B => A - B
	Mul, // A, B => A * B
	Div, // A, B => A / B
	Pow, // A, B => A ^ B -- Raises A to the power B
	Func(FuncType), // A => func(A)
	Root, // A, B => Ath root of B
	Comma, // NOP. Should not be in the final vector. Used to indicate a seperation between arguments in a function.
	ParenOpen, // NOP. Should not be in the final vector.
	ParenClose, // NOP. Should not be in the final vector.
}
impl Command {
	/// Number of numbers that this command pops from the stack
	pub fn pops(&self) -> usize {
		match self {
			&Com::Var(_) => 0,
			&Com::Int(_) => 0,
			&Com::Float(_) => 0,
			&Com::Add => 2,
			&Com::Sub => 2,
			&Com::Mul => 2,
			&Com::Div => 2,
			&Com::Pow => 2,
			&Com::Func(_) => 1,
			&Com::Root => 2,
			&Com::Comma => 0,
			&Com::ParenOpen => 0,
			&Com::ParenClose => 0,
		}
	}
	/// Number of numbers that this command pushes to the stack
	pub fn pushes(&self) -> usize {
		match self {
			&Com::Var(_) => 1,
			&Com::Int(_) => 1,
			&Com::Float(_) => 1,
			&Com::Add => 1,
			&Com::Sub => 1,
			&Com::Mul => 1,
			&Com::Div => 1,
			&Com::Pow => 1,
			&Com::Func(_) => 1,
			&Com::Root => 1,
			&Com::Comma => 0,
			&Com::ParenOpen => 0,
			&Com::ParenClose => 0,
		}
	}
	pub fn is_operator(&self) -> bool {
		match self {
			&Com::Add | &Com::Sub | &Com::Mul | &Com::Div | &Com::Pow | &Com::Func(_) | &Com::Root => true,
			&Com::Var(_) | &Com::Int(_) | &Com::Float(_) | &Com::Comma | &Com::ParenOpen | &Com::ParenClose => false
		}
	}
	pub fn prescedence(&self) -> Option<u32> {
		match self {
			&Com::Add | &Com::Sub => Some(1),
			&Com::Mul | &Com::Div => Some(2),
			&Com::Pow => Some(3),
			&Com::Func(_) | &Com::Root => Some(4),
			&Com::Var(_) | &Com::Int(_) | &Com::Float(_) | &Com::Comma | &Com::ParenOpen | &Com::ParenClose => None
		}
	}
	pub fn is_left_associative(&self) -> bool {
		match self {
			&Com::Add | &Com::Sub | &Com::Mul | &Com::Div | &Com::Func(_) | &Com::Root => true,
			&Com::Pow => false,
			&Com::Var(_) | &Com::Int(_) | &Com::Float(_) | &Com::Comma | &Com::ParenOpen | &Com::ParenClose => false,
		}
	}
	pub fn is_right_associative(&self) -> bool {
		match self {
			&Com::Add | &Com::Sub | &Com::Mul | &Com::Div | &Com::Func(_) | &Com::Root => false,
			&Com::Pow => true,
			&Com::Var(_) | &Com::Int(_) | &Com::Float(_) | &Com::Comma | &Com::ParenOpen | &Com::ParenClose => false,
		}
	}
}

/// Holds state
/// e.g. stack, variable states.
#[derive(Debug)]
pub struct VM {
	stack: Vec<f64>,
	vars: HashMap<char, f64>,
	num: usize, // number of commands executed on this VM
}
impl VM {
	pub fn new() -> VM {
		let mut hm = HashMap::new();
		hm.insert('π', M_PI);
		hm.insert('e', M_E);
		hm.insert('φ', M_GOLDEN_RATIO);
		VM{stack:Vec::new(), vars:hm, num:0}
	}
	#[inline(always)]
	pub fn push(&mut self, v: f64) {
		self.stack.push(v);
	}
	#[inline(always)]
	pub fn pop(&mut self) -> Option<f64> {
		self.stack.pop()
	}
	#[inline(always)]
	pub fn peek(&mut self) -> Option<f64> {
		self.stack.get(0).map(|f| {*f})
	}
	#[inline(always)]
	pub fn set_var(&mut self, id: char, v: f64) {
		self.vars.insert(id, v);
	}
	#[inline(always)]
	pub fn get_var(&mut self, id: char) -> f64 {
		*self.vars.get(&id).unwrap_or(&0.0)
	}
	#[inline(always)]
	pub fn clear_stack(&mut self) {
		self.stack.clear();
	}
	#[inline(always)]
	pub fn stack_size(&self) -> usize {
		self.stack.len()
	}
	pub fn get_result(&mut self, coms: &[Command]) -> Result<f64, ParseError> {
		try!(self.execute_all(coms));
		if self.stack_size() == 0 {
			Err(StackExhausted)
		} else if self.stack_size() > 1 {
			Err(SyntaxError)
		} else {
			Ok(self.stack[0])
		}
	}
	pub fn execute_all(&mut self, coms: &[Command]) -> Result<(), ParseError> {
		let debug_print: bool = unsafe { debug_print_stage3 };
		
		for com in coms.iter() {
			if debug_print {
				let mut vars_str = String::with_capacity(16);
				for (k, v) in self.vars.iter() {
					let _ = write!(vars_str, "{}:{} ", k, v);
				}
				vars_str.trim();
				let mut stack_str = String::with_capacity(32);
				for v in self.stack.iter() {
					let _ = write!(stack_str, "{} ", v);
				}
				stack_str.trim();
				
				println!("{: <20} | {: <12} | {}", vars_str, format!("{:?}", com), stack_str);
			}
			try!(self.execute(com));
		}
		Ok(())
	}
	pub fn execute(&mut self, com: &Command) -> Result<(), ParseError> {
		if self.stack_size() < com.pops() {
			return Err(StackExhausted);
		}
		match com {
			&Com::Var(id) => {
				let val = self.get_var(id);
				self.push(val);
			},
			&Com::Int(v) => self.push(v as f64),
			&Com::Float(v) => self.push(v),
			&Com::Add => {
				let b = self.pop().unwrap(); // Intentional B first.
				let a = self.pop().unwrap();
				self.push(a + b);
			},
			&Com::Sub => {
				let b = self.pop().unwrap(); // Intentional B first.
				let a = self.pop().unwrap();
				self.push(a - b);
			},
			&Com::Mul => {
				let b = self.pop().unwrap(); // Intentional B first.
				let a = self.pop().unwrap();
				self.push(a * b);
			},
			&Com::Div => {
				let b = self.pop().unwrap(); // Intentional B first.
				let a = self.pop().unwrap();
				self.push(a / b);
			},
			&Com::Pow => {
				let b = self.pop().unwrap(); // Intentional B first.
				let a = self.pop().unwrap();
				self.push(a.powf(b));
			},
			&Com::Func(ref func) => {
				let sqrt = func.execute(self.pop().unwrap());
				self.push(sqrt);
			},
			&Com::Root => {
				let b = self.pop().unwrap(); // Intentional B first.
				let a = self.pop().unwrap();
				self.push(b.powf(a.recip()));
			},
			&Com::Comma | &Com::ParenOpen | &Com::ParenClose => return Err(IllegalCommand(com.clone(), self.num)),
		}
		self.num += 1;
		Ok(())
	}
}

pub static mut debug_print_stage1: bool = false;
pub static mut debug_print_stage2: bool = false;
pub static mut debug_print_stage3: bool = false;

// Changes ex into a vector of commands to execute to get the value of the expression.
pub fn expr_to_commands(ex: VExprRef) -> Result<Vec<Command>, ParseError> {
	let mut infix = Vec::new();
	try!(expr_to_infix(ex, &mut infix));
	print!("infix  : ");
	print_commands(&infix, true);
	let postfix = try!(infix_to_postfix(&infix));
	print!("postfix: ");
	print_commands(&postfix, true);
	Ok(postfix)
}

#[allow(unused_assignments)]
fn expr_to_infix(ex: VExprRef, infix: &mut Vec<Command>) -> Result<(), ParseError> {
	let mut num_buf = String::new();
	
	let debug_print: bool = unsafe { debug_print_stage1 };
	
	if debug_print {
		println!("{: ^18} | {: ^25} | {: ^18} | {: ^20}", "infix", "tok", "last_tok", "num_buf");
		println!("-------------------|---------------------------|--------------------|---------------------");
	}
	for tok in ex.borrow().tokens.iter() {
		let mut last_tok = if infix.len() >= 1 {
			infix.get(infix.len() - 1).map(|c| { c.clone() })
		} else {
			None
		};
		
		match tok {
			&VToken::Digit(ref dgt) => {
				num_buf.push(*dgt);
			},
			&VToken::Char('.') => {
				num_buf.push('.');
			},
			_ if num_buf.len() >= 1 => {
				// Flush buffer
				infix.push(try!(parse_num_buf(&num_buf)));
				num_buf.clear();
			},
			_ => {}
		}
		
		last_tok = if infix.len() >= 1 {
			infix.get(infix.len() - 1).map(|c| { c.clone() })
		} else {
			None
		};
		
		match tok {
			&VToken::Digit(_) => {}
			&VToken::Char(ref chr) => {
				match chr {
					&'(' => infix.push(Com::ParenOpen),
					&')' => infix.push(Com::ParenClose),
					&'.' => {},
					_ => {
						infix.push(Com::Var(*chr));
					},
				}
			},
			&VToken::Op(ref op) => {
				infix.push(match op {
					&OpType::Add => Com::Add,
					&OpType::Sub => Com::Sub,
					&OpType::Mul => Com::Mul,
					&OpType::Div => Com::Div,
				});
			},
			&VToken::Pow(ref inner_ex) => {
				infix.push(Com::Pow);
				infix.push(Com::ParenOpen);
				try!(expr_to_infix(inner_ex.clone(), infix));
				infix.push(Com::ParenClose);
			},
			&VToken::Root(ref degree_ex, ref inner_ex) => {
				infix.push(Com::Root);
				infix.push(Com::ParenOpen);
				try!(expr_to_infix(degree_ex.clone(), infix));
				infix.push(Com::Comma);
				try!(expr_to_infix(inner_ex.clone(), infix));
				infix.push(Com::ParenClose);
			},
			&VToken::Func(ref func, ref inner_ex) => {
				infix.push(Com::Func(func.clone()));
				infix.push(Com::ParenOpen);
				try!(expr_to_infix(inner_ex.clone(), infix));
				infix.push(Com::ParenClose);
			}
		}
		if debug_print { println!("{: >18} | {: <25} | {: <18} | {: <20}", commands_to_string(&infix, true), format!("{:?}", tok), format!("{:?}", last_tok), num_buf); }
	}
	if num_buf.len() >= 1 {
		// Flush buffer
		infix.push(try!(parse_num_buf(&num_buf)));
		num_buf.clear();
	}
	
	if debug_print {
		println!("({: ^25}, {: ^25})", "first", "second");
		println!("(-------------------------,--------------------------)");
	}
	
	let mut i = 1;
	while i < infix.len() {
		let com_pair = (infix.get(i - 1).unwrap().clone(), infix.get(i).unwrap().clone());
		if debug_print {
			println!("({: <25}, {: <25}) i = {}", format!("{:?}", com_pair.0).as_str(), format!("{:?}", com_pair.1).as_str(), i);
		}
		i += match com_pair {
			(Com::Var(_), Com::Var(_)) | (Com::Var(_), Com::Int(_)) | (Com::Int(_), Com::Var(_))
				| (Com::Float(_), Com::Var(_)) | (Com::Var(_), Com::Float(_)) => { infix.insert(i, Com::Mul); 1 },
			_ => { 0 }
		};
		
		i += 1;
	}
	
	// Check for errors
	Ok(())
}

fn parse_num_buf(num_buf: &str) -> Result<Command, ParseError> {
	// Flush buffer
	let com = if num_buf.find('.').is_some() {
		// Try parsing as float
		match num_buf.parse() {
			Ok(v) => Com::Float(v),
			Err(_) => return Err(FloatParseError),
		}
	} else {
		// Try parsing as int
		match num_buf.parse() {
			Ok(v) => Com::Int(v),
			Err(_) => return Err(OverflowError),
		}
	};
	
	Ok(com)
}

/// What follows is the "shunting yard algorithm" (https://en.wikipedia.org/wiki/Shunting-yard_algorithm)
fn infix_to_postfix(infix: &[Command]) -> Result<Vec<Command>, ParseError> {
	let mut postfix: Vec<Command> = Vec::new();
	let mut stack: Vec<Command> = Vec::new();
	
	let debug_print: bool = unsafe { debug_print_stage2 };
	
	if debug_print {
		println!("{: ^15} | {: ^18} | {: ^18}", "tok", "stack", "output");
		println!("----------------|--------------------|-------------------");
	}
	let mut i = 0;
	for tok in infix {
		match tok {
			&Com::Var(_) | &Com::Int(_) | &Com::Float(_) => postfix.push(tok.clone()),
			&Com::Func(_) => stack.push(tok.clone()),
			&Com::Comma => {
				loop {
					match stack.pop() {
						Some(Com::ParenOpen) => { stack.push(Com::ParenOpen); break; },
						Some(pop) => postfix.push(pop),
						None => return Err(UnmatchedParen),
					}
				}
			},
			&Com::ParenOpen => stack.push(Com::ParenOpen),
			&Com::ParenClose => {
				// Push all tokens from stack to output until left parenthesis
				loop {
					match stack.pop() {
						Some(Com::ParenOpen) => break,
						Some(v) => postfix.push(v),
						None => return Err(UnmatchedParen),
					}
				}
			},
			_ if tok.is_operator() => {
				loop {
					if stack.len() == 0 {
						break;
					}
					let peek = stack[stack.len() - 1].clone();
					if peek.is_operator() {
						if (tok.is_left_associative() && tok.prescedence() <= peek.prescedence()) || (tok.is_right_associative() && tok.prescedence() > peek.prescedence()) {
							stack.pop();
							postfix.push(peek.clone());
						} else {
							break;
						}
					} else {
						break;
					}
				}
				stack.push(tok.clone());
			},
			_ => return Err(IllegalCommand(tok.clone(), i))
		}
		if debug_print { println!("{: >15} | {: <18} | {: <18}", format!("{:?}", tok), commands_to_string(&stack, true), commands_to_string(&postfix, true)); }
		i += 1;
	}
	while stack.len() > 0 {
		let pop = stack.pop().unwrap();
		if pop == Com::ParenOpen || pop == Com::ParenClose {
			return Err(UnmatchedParen);
		}
		postfix.push(pop);
	}

	Ok(postfix)
}

pub fn print_commands(coms: &[Command], spaces: bool) {
	println!("{}", commands_to_string(coms, spaces));
}

pub fn commands_to_string(coms: &[Command], spaces: bool) -> String {
	let mut s = String::new();
	for com in coms.iter() {
		match com {
			&Com::Var(ref var) => s.push(*var),
			&Com::Int(ref i) => { let _ = write!(s, "{}", i); },
			&Com::Float(ref f) => { let _ = write!(s, "{}", f); },
			&Com::Add => s.push(CHAR_ADD),
			&Com::Sub => s.push(CHAR_SUB),
			&Com::Mul => s.push(CHAR_MUL),
			&Com::Div => s.push(CHAR_DIV),
			&Com::Pow => s.push_str("^"),
			&Com::Func(ref func) => { let _ = write!(s, "{}", *func); },
			&Com::Root => s.push_str("root"),
			&Com::Comma => s.push(','),
			&Com::ParenOpen => s.push('('),
			&Com::ParenClose => {
				if !spaces && s.len() != 0 && s.is_char_boundary(s.len() - 1) && s.char_at(s.len() - 1) == ' ' {
					s.pop();
				}
				s.push(')');
			}
		}
		if spaces {
			s.push(' ');
		}
	}
	s.trim();
	s
}

#[test]
fn commands_test() {
	test_command(&[Com::Int(5), Com::Int(3), Com::Int(2), Com::Add, Com::Mul], Some(25.0));
	test_command(&[Com::Int(5), Com::Int(10), Com::Div], Some(0.5));
	test_command(&[Com::Int(5), Com::Int(10), Com::Sub], Some(-5.0));
	test_command(&[Com::Int(5), Com::Int(2), Com::Pow], Some(25.0));
	test_command(&[Com::Int(5), Com::Int(3), Com::Pow], Some(25.0*5.0));
	test_command(&[Com::Int(25), Com::Sqrt], Some(5.0));
	test_command(&[Com::Int(3), Com::Int(25*5), Com::Root], Some(5.0));
}

#[allow(dead_code)]
fn test_command(coms: &[Command], expected_res: Option<f64>) {
	const ACCEPTABLE_ERROR: f64 = ::std::f64::EPSILON * 8.0;
	let res = VM::new().get_result(coms).ok();
	println!("{} == {:?}? (={:?})", commands_to_string(coms, true), expected_res, res);
	match (res, expected_res) {
		(None, None) => {},
		(Some(_), None) | (None, Some(_)) => panic!("testing assertion failed"),
		(Some(r), Some(e)) => {
			if (r - e).abs() > ACCEPTABLE_ERROR {
				panic!("testing assertion failed");
			}
		},
	}
}
