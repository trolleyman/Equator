/// Module for turning the equation into commands
use std::collections::HashMap;
use std::fmt::Write;

use vis::*;
use edit::*;
use func::*;
use err::*;

#[allow(non_snake_case)]
mod Com {
	pub use super::Command::*;
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Command {
	Var(char), // Pushes variable with char identifier to the stack
	Int(i64), // Pushes integer literal to the stack
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
	pub fn execute(&self, vm: &mut VM) -> Result<(), ParseError> {
		if vm.stack_size() < self.pops() {
			return Err(StackExhausted);
		}
		match self {
			&Com::Var(id) => {
				let val = vm.get_var(id);
				vm.push(val);
			},
			&Com::Int(v) => vm.push(v as f64),
			&Com::Add => {
				let b = vm.pop(); // Intentional B first.
				let a = vm.pop();
				vm.push(a + b);
			},
			&Com::Sub => {
				let b = vm.pop(); // Intentional B first.
				let a = vm.pop();
				vm.push(a - b);
			},
			&Com::Mul => {
				let b = vm.pop(); // Intentional B first.
				let a = vm.pop();
				vm.push(a * b);
			},
			&Com::Div => {
				let b = vm.pop(); // Intentional B first.
				let a = vm.pop();
				vm.push(a / b);
			},
			&Com::Pow => {
				let b = vm.pop(); // Intentional B first.
				let a = vm.pop();
				vm.push(a.powf(b));
			},
			&Com::Func(ref func) => {
				let sqrt = func.execute(vm.pop());
				vm.push(sqrt);
			},
			&Com::Root => {
				let b = vm.pop(); // Intentional B first.
				let a = vm.pop();
				vm.push(b.powf(a.recip()));
			},
			&Com::Comma      => return Err(IllegalChar),
			&Com::ParenOpen  => return Err(IllegalChar),
			&Com::ParenClose => return Err(IllegalChar),
		}
		Ok(())
	}
	/// Number of numbers that this command pops from the stack
	pub fn pops(&self) -> usize {
		match self {
			&Com::Var(_) => 0,
			&Com::Int(_) => 0,
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
			&Com::Var(_) | &Com::Int(_) | &Com::Comma | &Com::ParenOpen | &Com::ParenClose => false
		}
	}
	pub fn prescedence(&self) -> Option<u32> {
		match self {
			&Com::Add | &Com::Sub => Some(1),
			&Com::Mul | &Com::Div => Some(2),
			&Com::Pow => Some(3),
			&Com::Func(_) | &Com::Root => Some(4),
			&Com::Var(_) | &Com::Int(_) | &Com::Comma | &Com::ParenOpen | &Com::ParenClose => None
		}
	}
	pub fn is_left_associative(&self) -> bool {
		match self {
			&Com::Add | &Com::Sub | &Com::Mul | &Com::Div | &Com::Func(_) | &Com::Root => true,
			&Com::Pow => false,
			&Com::Var(_) | &Com::Int(_) | &Com::Comma | &Com::ParenOpen | &Com::ParenClose => false,
		}
	}
	pub fn is_right_associative(&self) -> bool {
		match self {
			&Com::Add | &Com::Sub | &Com::Mul | &Com::Div | &Com::Func(_) | &Com::Root => false,
			&Com::Pow => true,
			&Com::Var(_) | &Com::Int(_) | &Com::Comma | &Com::ParenOpen | &Com::ParenClose => false,
		}
	}
}

/// Holds state
/// e.g. stack, variable states.
#[derive(Debug)]
pub struct VM {
	stack: Vec<f64>,
	vars: HashMap<char, f64>,
}
impl VM {
	pub fn new() -> VM {
		VM{stack:Vec::new(), vars:HashMap::new()}
	}
	#[inline(always)]
	pub fn push(&mut self, v: f64) {
		self.stack.push(v);
	}
	#[inline(always)]
	pub fn pop(&mut self) -> f64 {
		self.stack.pop().expect("VM error: stack exhausted")
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
	pub fn stack_size(&self) -> usize {
		self.stack.len()
	}
}

// Changes ex into a vector of commands to execute to get the value of the expression.
pub fn expr_to_commands(ex: VExprRef) -> Result<Vec<Command>, ParseError> {
	let mut infix = Vec::new();
	try!(expr_to_infix(ex, &mut infix));
	print!("infix  : ");
	print_commands(&infix);
	let postfix = try!(infix_to_postfix(&infix));
	print!("postfix: ");
	print_commands(&postfix);
	Ok(postfix)
}

fn expr_to_infix(ex: VExprRef, infix: &mut Vec<Command>) -> Result<(), ParseError> {
	let mut num_buf = String::new();
	for tok in ex.borrow().tokens.iter() {
		match tok {
			&VToken::Char(ref chr) if chr.is_digit(10) => {
				num_buf.push(*chr);
				continue;
			}
			_ if num_buf.len() >= 1 => {
				// Flush buffer
				infix.push(Com::Int(num_buf.parse().unwrap()));
				num_buf.clear();
			},
			_ => {}
		}
		
		match tok {
			&VToken::Char(ref chr) => {
				infix.push(match chr {
					&CHAR_ADD => Com::Add,
					&CHAR_SUB => Com::Sub,
					&CHAR_MUL => Com::Mul,
					&CHAR_DIV => Com::Div,
					&'(' => Com::ParenOpen,
					&')' => Com::ParenClose,
					_ => Com::Var(*chr),
				});
			}, // Shouldn't be numeric
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
	}
	if num_buf.len() >= 1 {
		infix.push(Com::Int(num_buf.parse().unwrap()));
	}
	Ok(())
}
/// What follows is the "shunting yard algorithm" (https://en.wikipedia.org/wiki/Shunting-yard_algorithm)
fn infix_to_postfix(infix: &[Command]) -> Result<Vec<Command>, ParseError> {
	let mut postfix: Vec<Command> = Vec::new();
	let mut stack: Vec<Command> = Vec::new();
	
	println!("tok | stack | output");
	for tok in infix {
		match tok {
			&Com::Var(_) | &Com::Int(_) => postfix.push(tok.clone()),
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
			_ => return Err(IllegalChar)
		}
		println!("{:?} \t| {} \t| {}", tok, commands_to_string(&stack), commands_to_string(&postfix));
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

pub fn execute_commands_with_vm(coms: &[Command], vm: &mut VM) -> Result<(), ParseError> {
	for com in coms.iter() {
		try!(com.execute(vm));
	}
	Ok(())
}

pub fn execute_commands(coms: &[Command]) -> Result<f64, ParseError> {
	let mut vm = VM::new();
	try!(execute_commands_with_vm(coms, &mut vm));
	if vm.stack_size() == 0 {
		Err(StackExhausted)
	} else {
		Ok(vm.pop())
	}
}

pub fn print_commands(coms: &[Command]) {
	println!("{}", commands_to_string(coms));
}

pub fn commands_to_string(coms: &[Command]) -> String {
	let mut s = String::new();
	for com in coms.iter() {
		match com {
			&Com::Var(ref var) => { s.push(*var); s.push(' ') },
			&Com::Int(ref i) => { let _ = write!(s, "{} ", i); },
			&Com::Add => { s.push(CHAR_ADD); s.push(' '); },
			&Com::Sub => { s.push(CHAR_SUB); s.push(' '); },
			&Com::Mul => { s.push(CHAR_MUL); s.push(' '); },
			&Com::Div => { s.push(CHAR_DIV); s.push(' '); },
			&Com::Pow => s.push_str("^ "),
			&Com::Func(ref func) => s.push_str(&format!("{} ", *func)),
			&Com::Root => s.push_str("root "),
			&Com::Comma => s.push_str(", "),
			&Com::ParenOpen => s.push('('),
			&Com::ParenClose => {
				if s.len() != 0 && s.is_char_boundary(s.len() - 1) && s.char_at(s.len() - 1) == ' ' {
					s.pop();
				}
				s.push(')');
			}
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
	let res = execute_commands(coms).ok();
	println!("{} == {:?}? (={:?})", commands_to_string(coms), expected_res, res);
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