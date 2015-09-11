/// Module for turning the equation into commands
use std::collections::HashMap;
use std::fmt::{Display, Formatter, Write, self};
use consts::*;
use vis::*;
use func::*;
use err::*;
use edit;
use num::Num;

#[allow(non_snake_case)]
mod Com {
	pub use super::Command::*;
}
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Command {
	Var(char), // Pushes variable with char identifier to the stack
	Num(Num), // Pushes literal to the stack
	Add, // A, B => A + B
	Sub, // A, B => A - B
	Mul, // A, B => A * B
	Div, // A, B => A / B
	Neg, // A => - A
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
			&Com::Num(_) => 0,
			&Com::Add => 2,
			&Com::Sub => 2,
			&Com::Mul => 2,
			&Com::Div => 2,
			&Com::Neg => 1,
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
			&Com::Num(_) => 1,
			&Com::Add => 1,
			&Com::Sub => 1,
			&Com::Mul => 1,
			&Com::Div => 1,
			&Com::Neg => 1,
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
			&Com::Add | &Com::Sub | &Com::Mul | &Com::Div | &Com::Neg | &Com::Pow | &Com::Func(_) | &Com::Root => true,
			&Com::Var(_) | &Com::Num(_) | &Com::Comma | &Com::ParenOpen | &Com::ParenClose => false
		}
	}
	pub fn prescedence(&self) -> Option<u32> {
		match self {
			&Com::Pow => Some(1),
			&Com::Add | &Com::Sub => Some(2),
			&Com::Mul | &Com::Div => Some(3),
			&Com::Neg => Some(4),
			&Com::Func(_) | &Com::Root => Some(5),
			&Com::Var(_) | &Com::Num(_) | &Com::Comma | &Com::ParenOpen | &Com::ParenClose => None
		}
	}
	pub fn is_left_associative(&self) -> bool {
		match self {
			&Com::Add | &Com::Sub | &Com::Mul | &Com::Div | &Com::Func(_) | &Com::Root | &Com::Neg => true,
			&Com::Pow => false,
			&Com::Var(_) | &Com::Num(_) | &Com::Comma | &Com::ParenOpen | &Com::ParenClose => false,
		}
	}
	pub fn is_right_associative(&self) -> bool {
		match self {
			&Com::Add | &Com::Sub | &Com::Mul | &Com::Div | &Com::Func(_) | &Com::Root | &Com::Neg => false,
			&Com::Pow => true,
			&Com::Var(_) | &Com::Num(_) | &Com::Comma | &Com::ParenOpen | &Com::ParenClose => false,
		}
	}
	/// If an implicit multiplication is performed if this command is on the left, and the other command is_automul_right()
	pub fn is_left_automul(&self) -> bool {
		match self {
			&Com::Var(_) | &Com::Num(_) | &Com::Func(_) | &Com::Root | &Com::ParenClose => true,
			&Com::Add | &Com::Sub | &Com::Mul | &Com::Div | &Com::Neg | &Com::Pow | &Com::Comma | &Com::ParenOpen => false
		}
	}
	/// If an implicit multiplication is performed if this command is on the right, and the other command is_automul_left()
	pub fn is_right_automul(&self) -> bool {
		match self {
			&Com::Var(_) | &Com::Num(_) | &Com::Func(_) | &Com::Root | &Com::ParenOpen => true,
			&Com::Add | &Com::Sub | &Com::Mul | &Com::Div | &Com::Pow | &Com::Neg | &Com::Comma | &Com::ParenClose => false
		}
	}
}
impl Display for Command {
	fn fmt(&self, f: &mut Formatter) -> fmt::Result {
		match self {
			&Com::Num(n) => write!(f, "Num({})", n),
			_ => write!(f, "{:?}", self),
		}
	}
}

/// Holds state
/// e.g. stack, variable states.
#[derive(Debug)]
pub struct VM {
	stack: Vec<Num>,
	vars : HashMap<char, Num>,
	num  : usize, // number of commands executed on this VM
	last_result: Result<Num, ParseError>,
}
impl VM {
	pub fn new() -> VM {
		let mut hm = HashMap::new();
		hm.insert('π', Num::PI);
		hm.insert('e', Num::E);
		hm.insert('φ', Num::GOLDEN_RATIO);
		VM{stack:Vec::new(), vars:hm, num:0, last_result:Err(LastResultNotInitialized)}
	}
	#[inline(always)]
	pub fn push(&mut self, v: Num) {
		self.stack.push(v);
	}
	#[inline(always)]
	pub fn pop(&mut self) -> Option<Num> {
		self.stack.pop()
	}
	#[inline(always)]
	pub fn peek(&mut self) -> Option<Num> {
		self.stack.get(0).map(|f| {*f})
	}
	#[inline(always)]
	pub fn set_var(&mut self, id: char, v: Num) {
		self.vars.insert(id, v);
	}
	#[inline(always)]
	pub fn get_var(&mut self, id: char) -> Option<Num> {
		self.vars.get(&id).map(|f: &Num| *f)
	}
	#[inline(always)]
	pub fn clear_stack(&mut self) {
		self.stack.clear();
	}
	#[inline(always)]
	pub fn stack_size(&self) -> usize {
		self.stack.len()
	}
	pub fn get_result(&mut self, coms: &[Command]) -> Result<Num, ParseError> {
		match self.execute_all(coms) {
			Ok(_) => {},
			Err(e) => {
				self.last_result = Err(e.clone());
				return Err(e);
			}
		}
		let res = if self.stack_size() == 0 {
			Err(StackExhausted(coms.len()))
		} else if self.stack_size() > 1 {
			Err(SyntaxError)
		} else {
			Ok(self.stack[0])
		};
		self.last_result = res.clone();
		res
	}
	#[inline(always)]
	pub fn get_last_result(&self) -> Result<Num, ParseError> {
		self.last_result.clone()
	}
	pub fn execute_all(&mut self, coms: &[Command]) -> Result<(), ParseError> {
		let debug_print: bool = unsafe { debug_print_stage3 };
		
		if debug_print {
			let mut vars_str = String::with_capacity(16);
			for (k, v) in self.vars.iter() {
				let _ = write!(vars_str, "{}:{} ", k, v);
			}
			vars_str.trim();
			println!("vars: {}", vars_str);
			println!("{: ^12} | {}", "command", "stack");
		}
		
		let mut i = 0;
		for com in coms.iter() {
			try!(self.execute(com, i));
			if debug_print {
				let mut stack_str = String::with_capacity(32);
				for v in self.stack.iter() {
					let _ = write!(stack_str, "{} ", v);
				}
				stack_str.trim();
				
				println!("{: <12} | {}", format!("{}", com), stack_str);
			}
			i += 1;
		}
		if debug_print {
			let mut stack_str = String::with_capacity(32);
			for v in self.stack.iter() {
				let _ = write!(stack_str, "{} ", v);
			}
			stack_str.trim();
		}
		Ok(())
	}
	fn execute(&mut self, com: &Command, pos: usize) -> Result<(), ParseError> {
		if self.stack_size() < com.pops() {
			return Err(StackExhausted(pos));
		}
		match com {
			&Com::Var(id) => {
				let val = match self.get_var(id) {
					Some(v) => v,
					None => return Err(UndefVar(id, pos)),
				};
				self.push(val);
			},
			&Com::Num(v) => self.push(v),
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
			&Com::Neg => {
				let a = self.pop().unwrap();
				self.push(-a);
			},
			&Com::Pow => {
				let b = self.pop().unwrap(); // Intentional B first.
				let a = self.pop().unwrap();
				self.push(a.pow(b));
			},
			&Com::Func(ref func) => {
				match func.execute(self.pop().unwrap()) {
					Some(res) => self.push(res),
					None => return Err(CommandExecuteError(com.clone(), pos))
				}
			},
			&Com::Root => {
				let b = self.pop().unwrap(); // Intentional B first.
				let a = self.pop().unwrap();
				match a.recip().map(|recip| b.pow(recip)) {
					Some(res) => self.push(res),
					_ => return Err(CommandExecuteError(com.clone(), pos))
				}
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
	let mut num_buf_start: usize = 0;
	let mut i = 0;
	for tok in ex.borrow().tokens.iter() {
		let mut last_tok = if infix.len() >= 1 {
			infix.get(infix.len() - 1).map(|c| { c.clone() })
		} else {
			None
		};
		
		match tok {
			&VToken::Digit(ref dgt) => {
				if num_buf.len() == 0 {
					num_buf_start = i;
				}
				num_buf.push(*dgt);
			},
			&VToken::Char('.') => {
				num_buf.push('.');
			},
			_ if num_buf.len() >= 1 => {
				// Flush buffer
				infix.push(try!(parse_num_buf(&num_buf, &edit::Cursor::new_ex(ex.clone(), num_buf_start))));
				num_buf.clear();
			},
			_ => {}
		}
		
		last_tok = if infix.len() >= 1 {
			infix.get(infix.len() - 1).map(|c| { c.clone() })
		} else {
			None
		};
		
		let prev_tok = if i != 0 {
			ex.borrow().tokens.get(i - 1).map(|c| { c.clone() })
		} else {
			None
		};
		
		match tok {
			&VToken::Space => {
				return Err(IllegalToken(VToken::Space, edit::Cursor::new_ex(ex.clone(), i)));
			},
			&VToken::Digit(_) => {},
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
					&OpType::Sub => {
						if prev_tok.is_none() || (match prev_tok.unwrap() { VToken::Op(_) => true, _ => false }) {
							Com::Neg
						} else {
							Com::Sub
						}
					},
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
			},
			&VToken::Frac(ref num_ex, ref den_ex) => {
				infix.push(Com::ParenOpen);
				infix.push(Com::ParenOpen);
				try!(expr_to_infix(num_ex.clone(), infix));
				infix.push(Com::ParenClose);
				infix.push(Com::Div);
				infix.push(Com::ParenOpen);
				try!(expr_to_infix(den_ex.clone(), infix));
				infix.push(Com::ParenClose);
				infix.push(Com::ParenClose);
			}
		}
		if debug_print { println!("{: >18} | {: <25} | {: <18} | {: <20}", commands_to_string(&infix, true), format!("{:?}", tok), format!("{:?}", last_tok), num_buf); }
		i += 1;
	}
	if num_buf.len() >= 1 {
		// Flush buffer
		infix.push(try!(parse_num_buf(&num_buf, &edit::Cursor::new_ex(ex.clone(), num_buf_start))));
		num_buf.clear();
	}
	
	if debug_print {
		println!("");
		println!("|{: ^25}, {: ^25}| should_automul", "first", "second");
		println!("|-------------------------,--------------------------|");
	}
	
	let mut i = 1;
	while i < infix.len() {
		let com_pair = (infix.get(i - 1).unwrap().clone(), infix.get(i).unwrap().clone());
		if debug_print {
			println!("|{: <25}, {: <25}| {: <5} i = {}", format!("{:?}", com_pair.0).as_str(), format!("{:?}", com_pair.1).as_str(), should_automul(com_pair.0.clone(), com_pair.1.clone()).ok().unwrap_or(false), i);
		}
		
		match should_automul(com_pair.0, com_pair.1) {
			Ok(true) => {
				infix.insert(i, Com::Mul);
				i += 1;
			},
			Ok(false) => {},
			Err(e) => return Err(e),
		}
		i += 1;
	}
	
	// Check for errors
	Ok(())
}
fn should_automul(left: Command, right: Command) -> Result<bool, ParseError> {
	if right == Com::ParenOpen { // TODO: Add some more cases here
		if let Com::Func(_) = left {
			return Ok(false);
		} else if left == Com::Root {
			return Ok(false);
		} else if left == Com::Comma {
			return Err(SyntaxError);
		}
	}
	Ok(left.is_left_automul() && right.is_right_automul())
}

fn parse_num_buf(num_buf: &str, start: &edit::Cursor) -> Result<Command, ParseError> {
	// Flush buffer
	let com = match num_buf.parse() {
		Ok(v) => Com::Num(v),
		Err(_) => return Err(NumParseError(start.ex.clone(), start.pos, start.pos + num_buf.len() - 1)),
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
			&Com::Var(_) | &Com::Num(_) => postfix.push(tok.clone()),
			&Com::Func(_) => stack.push(tok.clone()),
			&Com::Comma => {
				loop {
					match stack.pop() {
						Some(Com::ParenOpen) => { stack.push(Com::ParenOpen); break; },
						Some(pop) => postfix.push(pop),
						None => return Err(UnmatchedParen(i)),
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
						None => return Err(UnmatchedParen(i)),
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
			return Err(UnmatchedParen(i));
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
			&Com::Num(ref v) => { let _ = write!(s, "{}", v); },
			&Com::Add => s.push(CHAR_ADD),
			&Com::Sub => s.push(CHAR_SUB),
			&Com::Mul => s.push(CHAR_MUL_SIMPLE),
			&Com::Div => s.push(CHAR_DIV),
			&Com::Neg => s.push_str("neg"),
			&Com::Pow => s.push('^'),
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
	fn test_one(coms: &[Command], expected: Option<Num>) {
		let res = VM::new().get_result(coms).ok();
		print!("{} = ", commands_to_string(coms, true));
		if res.is_some() {
			print!("Some({}) (", res.unwrap());
		} else {
			print!("None (");
		}
		if expected.is_some() {
			print!("Some({})) ? ", expected.unwrap());
		} else {
			print!("None) ? ");
		}
		println!("{}", res == expected);
		assert_eq!(res, expected);
	}
	
	test_one(&[Com::Num(Num::new(5, 0)), Com::Num(Num::new(3, 0)), Com::Num(Num::new(2, 0)), Com::Add, Com::Mul], Some(Num::new(25, 0)));
	test_one(&[Com::Num(Num::new(5, 0)), Com::Num(Num::new(10, 0)), Com::Div], Some(Num::new(5, -1)));
	test_one(&[Com::Num(Num::new(5, 0)), Com::Num(Num::new(10, 0)), Com::Sub], Some(Num::new(-5, 0)));
	test_one(&[Com::Num(Num::new(5, 0)), Com::Num(Num::new(2, 0)), Com::Pow], Some(Num::new(25, 0)));
	test_one(&[Com::Num(Num::new(5, 0)), Com::Num(Num::new(3, 0)), Com::Pow], Some(Num::new(5*5*5, 0)));
	test_one(&[Com::Num(Num::new(25, 0)), Com::Func(FuncType::Sqrt)], Some(Num::new(5, 0)));
	//test_one(&[Com::Num(Num::new(3, 0)), Com::Num(Num::new(5*5*5, 0)), Com::Root], Some(Num::new(5, 0)));
}
