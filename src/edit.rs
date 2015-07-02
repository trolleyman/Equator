use std::iter::RandomAccessIterator;

use std::fmt;

use gdk::{key, EventKey, self};

use consts::prelude::*;
use gui;
use com;
use render;
use err::*;
use vis::*;
use func::*;

pub struct Editor {
	pub root_ex: VExprRef,
	pub cursor: Cursor,
	pub vm: com::VM,
	pub extents: render::Extents,
	pub highlit_extents: Vec<render::Extent>,
}

#[derive(Clone)]
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
}

impl Editor {
	pub fn new() -> Self {
		let ex: VExprRef = VExpr::new_ref();
		Editor::with_expression(ex, 0)
	}
	pub fn with_expression(ex: VExprRef, pos: usize) -> Self {
		Editor{ root_ex: ex.clone(), cursor: Cursor::new_ex(ex, pos), vm: com::VM::new(), extents: render::Extents::new(), highlit_extents: Vec::new() }
	}
	
	/// Handles the keypress given, inserting the key pressed at the cursor's position.
	/// Returns true if the key has been handled.
	pub fn handle_keypress(&mut self, e: &EventKey) -> bool {
		let mut unhandled = false;
		match e.keyval {
			key::Left => {
				self.cursor.move_left();
			},
			key::Right => {
				self.cursor.move_right();
			},
			key::Up => {
				self.cursor.move_up();
			},
			key::Down => {
				self.cursor.move_down();
			},
			key::F1 => unsafe {
				com::debug_print_stage1 = !com::debug_print_stage1;
				if com::debug_print_stage1 { println!("command debug printing stage 1 (expr->infix) on."); 
				} else {                     println!("command debug printing stage 1 (expr->infix) off."); }
			},
			key::F2 => unsafe {
				com::debug_print_stage2 = !com::debug_print_stage2;
				if com::debug_print_stage2 { println!("command debug printing stage 2 (infix->postfix) on."); 
				} else {                     println!("command debug printing stage 2 (infix->postfix) off."); }
			},
			key::F3 => unsafe {
				com::debug_print_stage3 = !com::debug_print_stage3;
				if com::debug_print_stage3 { println!("command debug printing stage 3 (calculation) on."); 
				} else {                     println!("command debug printing stage 3 (calculation) off."); }
			},
			key::Delete => {self.cursor.delete();},
			key::BackSpace => {self.cursor.backspace();},
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
	pub fn insert_token(&mut self, tok: VToken) -> Result<(), ParseError> {
		match tok {
			VToken::Pow(_) => {
				// If there is a VToken::Pow(_) just before the token, don't insert it.
				let mut cursor_ex = self.cursor.ex.borrow_mut();
				if self.cursor.pos != 0 && match cursor_ex.tokens.get(cursor_ex.tokens.len() - 1) { Some(&VToken::Pow(_)) => true, _ => false } {
					Err(IllegalToken(tok, self.cursor.ex.clone(), self.cursor.pos))
				} else {
					cursor_ex.tokens.insert(self.cursor.pos, tok);
					Ok(())
				}
			},
			_ => {
				self.cursor.ex.borrow_mut().tokens.insert(self.cursor.pos, tok);
				Ok(())
			}
		}
	}
	
	/// Handles a click at the position (x, y), relative the the top left corner of the DrawingArea.
	/// Returns if the expression should be updated.
	pub fn handle_click(&mut self, x: f64, y: f64) -> bool {
		let mut dirty = self.highlit_extents.len() > 0;
		self.highlit_extents.clear();
		let mut sel_area = INFINITY;
		let mut selection = self.cursor.clone();
		
		for &(ex, ref cur) in self.extents.extents.iter() {
			if ex.contains(x, y) {
				dirty = true;
				//self.highlit_extents.push(ex);
				let ex_area = ex.w() * ex.h();
				if ex_area < sel_area {
					sel_area = ex_area;
					selection = cur.clone();
				}
			}
		}
		
		self.cursor = selection.clone();
		
		dirty
	}
	
	/// Returns true if the button press has been handled.
	#[allow(unused_mut)]
	pub fn handle_button_click(&mut self, id: gui::ButtonID) -> bool {
		let mut unhandled = false;
		match id {
			gui::ButtonID::Null => {}
			gui::ButtonID::Pow => {
				// Insert ^
				let inner_ref = VExpr::with_parent(self.cursor.ex.clone()).to_ref();
				let exp = VToken::Pow(inner_ref.clone());
				
				match self.insert_token(exp) {
					Ok(_) => {
						// Move cursor inside
						self.cursor.ex = inner_ref;
						self.cursor.pos = 0;
					},
					Err(e) => println!("error: {}", e),
				}
			},
			gui::ButtonID::Square => {
				// ^2
				let inner_ref = VExpr::with_parent(self.cursor.ex.clone()).to_ref();
				inner_ref.borrow_mut().tokens.push(VToken::Digit('2'));
				let exp = VToken::Pow(inner_ref.clone());
				
				match self.insert_token(exp) {
					Ok(_) => {
						// Move cursor inside
						self.cursor.ex = inner_ref;
						self.cursor.pos = 0;
					},
					Err(e) => println!("error: {}", e),
				}
			},
			gui::ButtonID::E => {
				// e^blank
				let inner_ref = VExpr::with_parent(self.cursor.ex.clone()).to_ref();
				let exp = VToken::Pow(inner_ref.clone());
				
				match self.insert_token(exp).and(self.insert_token(VToken::Char('e'))) {
					Ok(_) => {
						// Move cursor inside
						self.cursor.ex = inner_ref;
						self.cursor.pos = 0;
					},
					Err(e) => println!("error: {}", e),
				}
			},
			gui::ButtonID::Ln => {
				self.insert_func(FuncType::Ln);
			},
			gui::ButtonID::Sin => {
				self.insert_func(FuncType::Sin);
			},
			gui::ButtonID::Cos => {
				self.insert_func(FuncType::Cos);
			},
			gui::ButtonID::Tan => {
				self.insert_func(FuncType::Tan);
			},
			gui::ButtonID::Arsin => {
				self.insert_func(FuncType::Arsin);
			},
			gui::ButtonID::Arcos => {
				self.insert_func(FuncType::Arcos);
			},
			gui::ButtonID::Artan => {
				self.insert_func(FuncType::Artan);
			},
			gui::ButtonID::Sinh => {
				self.insert_func(FuncType::Sinh);
			},
			gui::ButtonID::Cosh => {
				self.insert_func(FuncType::Cosh);
			},
			gui::ButtonID::Tanh => {
				self.insert_func(FuncType::Tanh);
			},
			gui::ButtonID::Arsinh => {
				self.insert_func(FuncType::Arsinh);
			},
			gui::ButtonID::Arcosh => {
				self.insert_func(FuncType::Arcosh);
			},
			gui::ButtonID::Artanh => {
				self.insert_func(FuncType::Artanh);
			},
			gui::ButtonID::Sqrt => {
				self.insert_func(FuncType::Sqrt);
			},
			gui::ButtonID::Abs => {
				self.insert_func(FuncType::Abs);
			},
			gui::ButtonID::Fact => {
				self.insert_func(FuncType::Fact);
			}
			gui::ButtonID::Frac => {
				// Insert ^
				let num_ref = VExpr::with_parent(self.cursor.ex.clone()).to_ref();
				let den_ref = VExpr::with_parent(self.cursor.ex.clone()).to_ref();
				let frac = VToken::Frac(den_ref.clone(), num_ref.clone());
				
				match self.insert_token(frac) {
					Ok(_) => {
						// Move cursor inside
						self.cursor.ex = num_ref;
						self.cursor.pos = 0;
					},
					Err(e) => println!("error: {}", e),
				}
			},
			gui::ButtonID::Cbrt => {
				// Produce cube root (∛)
				let inner_ref = VExpr::with_parent(self.cursor.ex.clone()).to_ref();
				let degree_ref = VExpr::with_parent(self.cursor.ex.clone()).to_ref();
				degree_ref.borrow_mut().tokens.push(VToken::Digit('3'));
				let root = VToken::Root(degree_ref.clone(), inner_ref.clone());
				
				match self.insert_token(root) {
					Ok(_) => {
						// Move cursor inside
						self.cursor.ex = inner_ref;
						self.cursor.pos = 0;
					},
					Err(e) => println!("error: {}", e),
				}
			},
			gui::ButtonID::Var(id) => {
				if gui::get_gui_state() == gui::GuiState::Store {
					let res = self.vm.get_last_result();
					if res.is_ok() {
						self.vm.set_var(id, res.ok().unwrap());
					}
					gui::set_gui_state(gui::GuiState::Normal);
				} else {
					if self.insert_token(VToken::Char(id)).is_ok() {
						self.cursor.pos += 1;
					}
				}
			},
			gui::ButtonID::Const(id) => {
				if self.insert_token(VToken::Char(id)).is_ok() {
					self.cursor.pos += 1;
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
		let inner_ref = VExpr::with_parent(self.cursor.ex.clone()).to_ref();
		let func = VToken::Func(func, inner_ref.clone());
		
		if self.insert_token(func).is_ok() {
			// Move cursor inside
			self.cursor.ex = inner_ref;
			self.cursor.pos = 0;
		}
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
			'a' ... 'z' | 'A' ... 'Z' => {
				if gui::get_gui_state() == gui::GuiState::Store && c != 'e' {
					// Store the last result in the variable
					match self.vm.get_last_result() {
						Ok(val) => self.vm.set_var(c, val),
						Err(_) => {},
					}
					gui::set_gui_state(gui::GuiState::Normal);
					false
				} else {
					if self.insert_token(VToken::Char(c)).is_ok() {
						self.cursor.pos += 1;
						true
					} else {
						false
					}
				}
			},
			'(' | ')' | '.' => {
				if self.insert_token(VToken::Char(c)).is_ok() {
					self.cursor.pos += 1;
					true
				} else { false }
			},
			_ if c.is_digit(10) => {
				if self.insert_token(VToken::Digit(c)).is_ok() {
					self.cursor.pos += 1;
					true
				} else { false }
			},
			'+' => {
				if self.insert_token(VToken::Op(OpType::Add)).is_ok() {
					self.cursor.pos += 1;
					true
				} else { false }
			},
			'-' | CHAR_SUB => {
				if self.insert_token(VToken::Op(OpType::Sub)).is_ok() {
					self.cursor.pos += 1;
					true
				} else { false }
			},
			'*' | CHAR_MUL => {
				if self.insert_token(VToken::Op(OpType::Mul)).is_ok() {
					self.cursor.pos += 1;
					true
				} else { false }
			},
			'/' | CHAR_DIV => {
				if self.insert_token(VToken::Op(OpType::Div)).is_ok() {
					self.cursor.pos += 1;
					true
				} else { false }
			},
			'^' => {
				// Insert ^()
				let inner_ref = VExpr::with_parent(self.cursor.ex.clone()).to_ref();
				let pow = VToken::Pow(inner_ref.clone());
				
				if self.insert_token(pow).is_ok() {
					// Move cursor inside
					self.cursor.ex = inner_ref;
					self.cursor.pos = 0;
					true
				} else { false }
			},
			_ => false
		}
	}
	
	/// Debug prints the editor's state to the screen.
	pub fn print(&self) {
		match self.to_string() {
			Ok(s)  => println!("expr   : {}", s),
			Err(e) => println!("expr   : {:?}", e),
		}
	}
	/// Gets the editor's expression as a string.
	pub fn to_string(&self) -> Result<String, fmt::Error> {
		let mut s = String::new();
		try!(display_vexpr(self.root_ex.clone(), &Some(self.cursor.clone()), &mut s));
		Ok(s)
	}
}
