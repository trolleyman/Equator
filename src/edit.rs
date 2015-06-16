use std::iter::RandomAccessIterator;

use std::fmt::Write;

use gdk::{key, EventKey, self};

use consts::*;
use gui;
use com;
use vis::*;
use func::*;

pub struct Editor {
	pub root_ex: VExprRef,
	pub ex: VExprRef,
	pub pos: usize,
	pub vm: com::VM,
}

impl Editor {
	pub fn new() -> Self {
		let ex: VExprRef = VExpr::new_ref();
		Editor::with_expression(ex, 0)
	}
	pub fn with_expression(ex: VExprRef, pos: usize) -> Self {
		Editor{ root_ex:ex.clone(), ex:ex, pos:pos, vm:com::VM::new()}
	}
	
	/// Handles the keypress given, inserting the key pressed at the cursor's position.
	/// Returns true if the key has been handled.
	pub fn handle_keypress(&mut self, e: &EventKey) -> bool {
		let mut unhandled = false;
		match e.keyval {
			key::Left => {
				self.move_left();
			},
			key::Right => {
				self.move_right();
			},
			key::Up | key::Down => {
				// Not used for anything currently, handled to stop changing focus of buttons.
			},
			key::F1 => unsafe {
				com::debug_print_stage1 = !com::debug_print_stage1;
				if com::debug_print_stage1 { println!("command debug printing stage 1 on."); 
				} else {                     println!("command debug printing stage 1 off."); }
			},
			key::F2 => unsafe {
				com::debug_print_stage2 = !com::debug_print_stage2;
				if com::debug_print_stage2 { println!("command debug printing stage 2 on."); 
				} else {                     println!("command debug printing stage 2 off."); }
			},
			key::F3 => unsafe {
				com::debug_print_stage3 = !com::debug_print_stage3;
				if com::debug_print_stage3 { println!("command debug printing stage 3 on."); 
				} else {                     println!("command debug printing stage 3 off."); }
			},
			key::Delete => {self.delete();},
			key::BackSpace => {self.backspace();},
			_ => {
				if let Some(c) = gdk::keyval_to_unicode(e.keyval) {
					unhandled = !self.insert_char(c);
				} else {
					unhandled = true;
				}
			}
		}
		if !unhandled {
			gui::dirty_expression();
		}
		return !unhandled;
	}
	
	/// Inserts the token at `self.pos`.
	pub fn insert_token(&mut self, tok: VToken) {
		self.ex.borrow_mut().tokens.insert(self.pos, tok);
	}
	
	/// Returns true if the button press has been handled.
	#[allow(unused_mut)]
	pub fn handle_button_click(&mut self, id: gui::ButtonID) -> bool {
		let mut unhandled = false;
		match id {
			gui::ButtonID::Null => {}
			gui::ButtonID::Pow => {
				// Insert ^
				let inner_ref = VExpr::with_parent(self.ex.clone()).to_ref();
				let exp = VToken::Pow(inner_ref.clone());
				
				self.insert_token(exp);
				
				// Move cursor inside
				self.ex = inner_ref;
				self.pos = 0;
			},
			gui::ButtonID::Square => {
				// ^2
				let inner_ref = VExpr::with_parent(self.ex.clone()).to_ref();
				inner_ref.borrow_mut().tokens.push(VToken::Digit('2'));
				let exp = VToken::Pow(inner_ref.clone());
				
				self.insert_token(exp);
				
				// Move cursor inside
				self.pos += 1;
			},
			gui::ButtonID::Sin => {
				self.insert_func(FuncType::Sin);
			},
			gui::ButtonID::Arsin => {
				self.insert_func(FuncType::Arsin);
			},
			gui::ButtonID::Cos => {
				self.insert_func(FuncType::Cos);
			},
			gui::ButtonID::Arcos => {
				self.insert_func(FuncType::Arcos);
			},
			gui::ButtonID::Tan => {
				self.insert_func(FuncType::Tan);
			},
			gui::ButtonID::Artan => {
				self.insert_func(FuncType::Artan);
			},
			gui::ButtonID::Sqrt => {
				self.insert_func(FuncType::Sqrt);
			},
			gui::ButtonID::Cbrt => {
				// Produce cube root (∛)
				let inner_ref = VExpr::with_parent(self.ex.clone()).to_ref();
				let degree_ref = VExpr::with_parent(self.ex.clone()).to_ref();
				degree_ref.borrow_mut().tokens.push(VToken::Digit('3'));
				let root = VToken::Root(degree_ref.clone(), inner_ref.clone());
				
				self.insert_token(root);
				
				// Move cursor inside
				self.ex = inner_ref;
				self.pos = 0;
			},
			gui::ButtonID::Var(id) => {
				if gui::get_gui_state() == gui::GuiState::Store {
					let peek = self.vm.peek();
					println!("id({:?}), peek: {:?}", id, peek);
					if peek.is_some() {
						self.vm.set_var(id, peek.unwrap());
					}
					gui::set_gui_state(gui::GuiState::Normal);
				} else {
					self.insert_token(VToken::Char(id));
					self.pos += 1;
				}
			}
		}
		
		if unhandled {
			println!("button clicked (unhandled): {:?}", id);
		} else {
			println!("button clicked (  handled): {:?}", id);
			gui::dirty_expression();
		}
		
		return true;
	}
	
	pub fn insert_func(&mut self, func: FuncType) {
		let inner_ref = VExpr::with_parent(self.ex.clone()).to_ref();
		let func = VToken::Func(func, inner_ref.clone());
		
		self.insert_token(func);
		
		// Move cursor inside
		self.ex = inner_ref;
		self.pos = 0;
	}
	
	/// Inserts the text at `pos` in the expression `ex`.
	///
	/// Returns true if at least one character in teh string has been inserted.
	pub fn insert_str(&mut self, s: &str) -> bool {
		let mut update = false;
		for c in s.chars() {
			update = if self.insert_char(c) {true} else {update}
		}
		update
	}
	
	/// Returns true if the character has been inserted
	pub fn insert_char(&mut self, c: char) -> bool {
		match c {
			'a' ... 'z' | 'A' ... 'Z' | '(' | ')' | '.' => {
				self.insert_token(VToken::Char(c));
				self.pos += 1;
				true
			},
			_ if c.is_digit(10) => {
				self.insert_token(VToken::Digit(c));
				self.pos += 1;
				true
			},
			'+' => {
				self.insert_token(VToken::Op(OpType::Add));
				self.pos += 1;
				true
			},
			'-' | CHAR_SUB => {
				self.insert_token(VToken::Op(OpType::Sub));
				self.pos += 1;
				true
			},
			'*' | CHAR_MUL => {
				self.insert_token(VToken::Op(OpType::Mul));
				self.pos += 1;
				true
			},
			'/' | CHAR_DIV => {
				self.insert_token(VToken::Op(OpType::Div));
				self.pos += 1;
				true
			},
			'^' => {
				// Insert ^()
				let inner_ref = VExpr::with_parent(self.ex.clone()).to_ref();
				let pow = VToken::Pow(inner_ref.clone());
				
				self.insert_token(pow);
				
				// Move cursor inside
				self.ex = inner_ref;
				self.pos = 0;
				true
			},
			_ => false
		}
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
			if self.move_up() {
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
			if !self.move_up() {
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
		if !self.move_down() {
			// If not possible, move forward.
			self.pos += 1;
			if self.pos > self.ex.borrow().tokens.len() {
				// If out of range, move up + forward.
				if !self.move_up() {
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
	pub fn move_down(&mut self) -> bool {
		let ex_clone = self.ex.clone();
		let ex = ex_clone.borrow();
		let tok = match ex.tokens.get(self.pos).clone() {
			Some(t) => t,
			None => return false
		};
		match tok.get_inner_expr().iter().idx(0) {
			Some(expr) => {
				self.ex = expr.clone();
				self.pos = 0;
				true
			},
			None => false
		}
	}
	
	/// Moves the cursor to the parent token of the current token. Returns true if the operation was successful.
	pub fn move_up(&mut self) -> bool {
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
	
	/// Debug prints the editor's state to the screen.
	pub fn print(&self) {
		println!("expr   : {}", self.to_string().trim());
	}
	/// Gets the editor's expression as a string.
	pub fn to_string(&self) -> String {
		self.expr_to_string(self.root_ex.clone())
	}
	
	/// Needs to be a function for recursion, and needs to be in Editor to access the cursor.
	fn expr_to_string(&self, ex_ref: VExprRef) -> String {
		let mut buffer = String::with_capacity(128);
		let cursor_in_ex: bool = unsafe {
			// Check if ex_ref and self.ex point to the same object
			ex_ref.as_unsafe_cell().get() == self.ex.as_unsafe_cell().get()
		};
		
		let len = ex_ref.borrow().tokens.len();
		for i in 0..len {
			if cursor_in_ex && self.pos == i {
				// Print cursor
				buffer.push('|');
			}
			
			match ex_ref.borrow().tokens[i].clone() {
				VToken::Digit(c) | VToken::Char(c) => {
					buffer.push(c);
				},
				VToken::Op(op) => {
					let _ = write!(buffer, "{}", op);
				},
				VToken::Pow(inner_ex_ref) => {
					// Recursive stuff yay!
					buffer.push_str("^(");
					buffer.push_str(self.expr_to_string(inner_ex_ref).as_str());
					buffer.push(')');
				},
				VToken::Func(func_type, inner_ex_ref) => {
					buffer.push_str(format!(" {}(", func_type).as_str());
					buffer.push_str(self.expr_to_string(inner_ex_ref).as_str());
					buffer.push(')');
				}
				VToken::Root(degree_ex, inner_ex) => {
					buffer.push_str(" root(");
					buffer.push_str(self.expr_to_string(degree_ex).as_str());
					buffer.push_str(", ");
					buffer.push_str(self.expr_to_string(inner_ex).as_str());
					buffer.push(')');
				}
			}
		}
		
		if cursor_in_ex && self.pos == ex_ref.borrow().tokens.len() {
			// Print cursor
			if len == 0 {
				buffer.push(CHAR_HLBOX);
			} else {
				buffer.push('|');
			}
		} else if len == 0 {
			buffer.push(CHAR_BOX);
		}
		buffer
	}
}
