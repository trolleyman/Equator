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
	pub hitboxes: Box<[(render::Extent, Cursor)]>,
}

impl Editor {
	pub fn new() -> Self {
		let ex: VExprRef = VExpr::new_ref();
		Editor::with_expression(ex, 0)
	}
	pub fn with_expression(ex: VExprRef, pos: usize) -> Self {
		Editor{ root_ex: ex.clone(), cursor: Cursor::new_ex(ex, pos), errors: Vec::new(), hitboxes: box [] }
	}
	
	pub fn update_hitboxes(&mut self, new_hbs: Box<[(render::Extent, Cursor)]>) {
		self.hitboxes = new_hbs;
	}
	
	/// Handles the keypress given, inserting the key pressed at the cursor's position.
	/// Returns true if the key has been handled.
	pub fn handle_keypress(&mut self, e: &EventKey) -> bool {
		let mut dirty_exp = false;
		let mut dirty_gui = true;
		
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
			key::F4 => {
				render::toggle_debug_view();
			},
			key::Delete => {
				self.cursor.delete();
				dirty_exp = true;
			},
			key::BackSpace => {
				self.cursor.backspace();
				dirty_exp = true;
			},
			key::Return => {
				gui::do_calc();
				dirty_exp = false;
			}
			_ => {
				if let Some(c) = gdk::keyval_to_unicode(e.keyval) {
					dirty_exp = self.insert_char(c);
				} else {
					dirty_gui = false;
				}
			}
		}
		if dirty_exp {
			gui::dirty_expression();
		} else if dirty_gui {
			gui::dirty_gui();
		}
		return dirty_exp || dirty_gui;
	}
	
	/// Inserts the token at `self.pos`.
	pub fn insert_token(&mut self, tok: VToken) -> Result<(), ()> {
		match tok {
			VToken::Pow(_) => {
				// If there is a VToken::Pow(_) just before the token, don't insert it.
				let mut cursor_ex = self.cursor.ex.borrow_mut();
				if self.cursor.pos != 0 && match cursor_ex.tokens.get(cursor_ex.tokens.len() - 1) { Some(&VToken::Pow(_)) => true, _ => false } {
					Err(())
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
		let mut sel_area = INFINITY;
		let mut dirty = false;
		let mut selection = self.cursor.clone();
		
		for &(ex, ref cur) in self.hitboxes.iter() {
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
		
		if dirty == true {
			println!("mouse click: cursor moved");
		} else {
			println!("mouse click: cursor not moved");
		}
		
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
					let res = ::get_vm().get_last_result();
					if res.is_ok() {
						::get_vm().set_var(id, res.ok().unwrap());
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
					match ::get_vm().get_last_result() {
						Ok(val) => ::get_vm().set_var(c, val),
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
			Ok((s, e))  => {
				println!("expr   : {}", s);
				println!("         {}", e);
			},
			Err(e) => println!("expr   : {:?}", e),
		}
	}
	/// Gets the editor's expression as a string.
	pub fn to_string(&self) -> Result<(String, String), fmt::Error> {
		let mut s = String::new();
		let mut e = String::new();
		try!(display_vexpr_errors(self.root_ex.clone(), &Some(self.cursor.clone()), &self.errors, &mut s, &mut e));
		Ok((s, e))
	}
}

fn get_errors(ex: &VExprRef, errs: &mut Vec<Span>) {
	// If empty, error
	if ex.borrow().tokens.len() == 0 {
		errs.push(Span::new(ex.clone(), 0, 0));
		return;
	}
	
	// Check each operator for valid inputs
	let mut brackets: Vec<usize> = Vec::with_capacity(16);
	let tokens = &ex.borrow().tokens;
	let mut i = 0;
	while i < tokens.len() {
		// Check inner expressions for errors too
		if tokens[i].has_inner_expr() {
			for ex in tokens[i].get_inner_expr().iter() {
				get_errors(ex, errs);
			}
		}
		
		match &tokens[i] {
			&VToken::Op(_) => {
				// Check the operator is valid at that position
				if i == 0 || !is_token_term_left(&tokens[i - 1]) || i == tokens.len() - 1 || !is_token_term_right(&tokens[i + 1]) {
					errs.push(Span::new(ex.clone(), i, i + 1));
				}
			},
			&VToken::Pow(_) => {
				// Check that there is a valid token before the token
				if i == 0 || !is_token_term_left(&tokens[i - 1]) {
					// Error
					errs.push(Span::new(ex.clone(), i, i + 1));
				}
			},
			&VToken::Char('(') => {
				brackets.push(i);
			},
			&VToken::Char(')') => {
				match brackets.pop() {
					Some(_) => {},
					None => {
						errs.push(Span::new(ex.clone(), i, i + 1));
					}
				}
			},
			_ => {},
		}
		i += 1
	}
	
	for pos in brackets.into_iter() {
		errs.push(Span::new(ex.clone(), pos, pos + 1));
	}
}



fn is_token_term_left(t: &VToken) -> bool {
	match t {
		&VToken::Space | &VToken::Char(')') | &VToken::Digit(_) | &VToken::Pow(_)
			| &VToken::Frac(_, _) | &VToken::Root(_, _)  => true,
		&VToken::Char(ref c) if *c != '(' => true,
		&VToken::Op(_) | &VToken::Func(_, _) => false,
		_ => false,
	}
}
fn is_token_term_right(t: &VToken) -> bool {
	match t {
		&VToken::Space | &VToken::Char('(') | &VToken::Digit(_) | &VToken::Pow(_)
			| &VToken::Frac(_, _) | &VToken::Root(_, _) | &VToken::Func(_, _) => true,
		&VToken::Char(ref c) if *c != ')' => true,
		&VToken::Op(_) => false,
		_ => false,
	}
}

/*pub enum VToken {
	Space,
	Char(char),
	Digit(char),
	Op(OpType),
	Pow(VExprRef),
	Frac(VExprRef, VExprRef), // (numerator, denominator)
	Root(VExprRef, VExprRef),
	Func(FuncType, VExprRef),
}*/
