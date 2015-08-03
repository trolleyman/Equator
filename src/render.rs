use gtk::traits::*;

use cairo::{Antialias, Context, FontOptions};
use cairo::enums::FontSlant;
use cairo::enums::FontWeight;
use cairo::enums::HintStyle;
use cairo::LineCap;

use edit::{Editor, Cursor};
use vis::*;
use self::Align::*;
use func::FuncType;

static debug_view_extents: bool = false;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum FinalAlignment {
	Central, // Central
	Equals,  // Central, with = in the middle
	Debug    // Eqn in top left, with 15 pix margin
}
const fn get_final_alignment() -> FinalAlignment {
	FinalAlignment::Equals
}

#[repr(packed)]
#[derive(Debug, Copy, Clone)]
pub struct Extent {
	x0: f64,
	y0: f64,
	x1: f64,
	y1: f64
}
impl Extent {
	pub fn new(ex: (f64, f64, f64, f64)) -> Extent {
		unsafe { ::std::mem::transmute(ex) }
	}
	pub fn w(&self) -> f64 {
		self.x1 - self.x0
	}
	pub fn h(&self) -> f64 {
		self.y1 - self.y0
	}
	pub fn enclosing(&self, other: &Extent) -> Extent {
		Extent{
			x0: self.x0.min(other.x0), // min x
			y0: self.y0.min(other.y0), // min y
			x1: self.x1.max(other.x1), // max x
			y1: self.y1.max(other.y1)  // max y
		}
	}
	pub fn translate(&self, x: f64, y: f64) -> Extent {
		Extent {
			x0:self.x0 + x,
			y0:self.y0 + y,
			x1:self.x1 + x,
			y1:self.y1 + y,
		}
	}
	/// Returns if the rectangle contains the point (x, y)
	pub fn contains(&self, x: f64, y: f64) -> bool {
		(x >= self.x0 && x </*=*/ self.x1) && (y >= self.y0 && y </*=*/ self.y1)
	}
	
	/// Splits the extent into two equal sized extents, one on the left, one on the right.
	/// Returns (left, right)
	pub fn split_lr(&self) -> (Extent, Extent) {
		let w  = self.w();
		let l = Extent{x0:self.x0, y0:self.y0, x1:self.x0 + w / 2.0, y1:self.y1};
		let r = Extent{x0:l.x0   , y0:self.y0, x1:self.x0 + w      , y1:self.y1};
		(l, r)
	}
}

const INIT_FONT_SIZE: f64 = 24.0;

#[derive(Copy, Clone)]
pub struct ExtentState {
	pub len: usize,
	pub cursor_set: bool,
}
pub struct Extents {
	pub states: Vec<ExtentState>,
	pub hitboxes: Vec<(Extent, Cursor)>, // Hitboxes
	pub cursor_extent: Option<Extent>
}
impl Extents {
	pub fn new() -> Extents {
		Extents { states: Vec::new(), hitboxes: Vec::new(), cursor_extent: None }
	}
	pub fn reset(&mut self) {
		self.states.clear();
		self.hitboxes.clear();
		self.cursor_extent = None;
	}
	pub fn get_state(&self) -> ExtentState {
		ExtentState{ len:self.hitboxes.len(), cursor_set: self.cursor_extent.is_some() }
	}
	pub fn push_state(&mut self) {
		let state = self.get_state();
		self.states.push(state);
	}
	pub fn push(&mut self, extent: Extent, cursor: Cursor) {
		self.hitboxes.push((extent, cursor));
	}
	pub fn pop_state(&mut self) {
		let prev_state = match self.states.pop() { Some(v) => v, _ => { println!("error: mismatching states"); return; } };
		self.hitboxes.truncate(prev_state.len);
		
		if !prev_state.cursor_set && self.cursor_extent.is_some() {
			self.cursor_extent = None;
		}
	}
	pub fn translate_from_to(&mut self, from: ExtentState, to: ExtentState, x: f64, y: f64) {
		if !from.cursor_set && to.cursor_set {
			self.cursor_extent = self.cursor_extent.map(|ex| ex.translate(x, y));
			// println!("translated cursor: {}, {}", x, y);
		}
		
		for i in from.len..to.len {
			self.hitboxes[i].0 = self.hitboxes[i].0.translate(x, y);
		}
	}
	pub fn translate(&mut self, x:f64, y:f64) {
		let last_state = match self.states.pop() { Some(v) => v, _ => { println!("error: mismatching states"); return; } };
		let this_state = self.get_state();
		
		self.translate_from_to(last_state, this_state, x, y);
	}
	/// bool is true when it is transforming the cursor
	pub fn transform_from_to<F>(&mut self, from: ExtentState, to: ExtentState, f: F) where F: Fn(Extent, bool) -> Extent {
		if !from.cursor_set && to.cursor_set {
			self.cursor_extent = self.cursor_extent.map(|ex| f(ex, true));
		}
		
		for i in from.len..to.len {
			self.hitboxes[i].0 = f(self.hitboxes[i].0, false);
		}
	}
	pub fn transform<F>(&mut self, f: F) where F: Fn(Extent, bool) -> Extent {
		let last_state = match self.states.pop() { Some(v) => v, _ => { println!("error: mismatching states"); return; } };
		let this_state = self.get_state();
		
		self.transform_from_to(last_state, this_state, f);
	}
}

pub struct Render<'a> {
	pub exts: Extents,
	//pub hl_exts: Vec<Extent>,
	pub c: &'a Context,
	pub prev_extent: Option<Extent>,
	pub root_ex: VExprRef,
	pub cursor: Cursor,
}

impl<'a> Render<'a> {
	pub fn new(c: &'a Context, ed: &Editor) -> Render<'a> {
		Render {exts: Extents::new(), c: c, prev_extent: None, root_ex: ed.root_ex.clone(), cursor: ed.cursor.clone()}
	}
	
	#[allow(unused_variables)]
	pub fn render(&mut self, alloc_w: f64, alloc_h: f64) -> Extent {
		self.c.select_font_face("CMU Serif", FontSlant::Normal, FontWeight::Normal);
		self.c.set_font_size(INIT_FONT_SIZE);
		self.c.set_antialias(Antialias::Best);
		let opt = FontOptions::new();
		opt.set_antialias(Antialias::Best);
		opt.set_hint_style(HintStyle::Medium);
		self.c.set_font_options(opt);
		self.c.identity_matrix();
		
		self.exts = Extents::new();
		
		self.exts.push_state();
		let root_clone = self.root_ex.clone();
		let left_extent = self.path_expr(root_clone);
		
		// Now path the ' = 12.512' or whatever if there is a result, else path ' = ?'
		let ret = ::get_vm().get_last_result();
		let s = match ret {
			Ok(val) => {
				const DP: usize = 7;
				let fact: f64 = (10.0f64).powi(DP as i32);
				let mut full_s = format!("{}", (val * fact).round() / fact);
				match full_s.find('.') {
					Some(i) => if i + DP + 1 <= full_s.len() { full_s.truncate(i + DP + 1) },
					None => {}
				}
				full_s
			},
			Err(_)  => "?".to_string(),
		};
		self.exts.push_state();
		
		let (x_before_eq, _) = self.c.get_current_point();
		let mut right_extent = self.path_str(" = ");
		let (x_after_eq, _) = self.c.get_current_point();
		let x_mid = ((x_before_eq + x_after_eq) / 2.0).floor();
		right_extent = right_extent.enclosing(&self.path_str(&s));
		
		let full_extent = left_extent.enclosing(&right_extent);
		
		self.exts.pop_state();
		
		let path = self.c.copy_path();
		
		// === ALIGN ===
		let (mut x, mut y) = match get_final_alignment() {
			FinalAlignment::Central => align(&full_extent, alloc_w/2.0, alloc_h/2.0, Mid),
			FinalAlignment::Equals  => align(&Extent{x0:x_mid, x1:x_mid, y0:full_extent.y0, y1:full_extent.y1}, alloc_w/2.0, alloc_h/2.0, Mid),
			FinalAlignment::Debug   => align(&full_extent, 30.0, 30.0, BotRight),
		};
		
		x = x.floor();
		y = y.floor();
		self.exts.translate(x, y);
		
		self.c.identity_matrix();
		self.c.new_path();
		if debug_view_extents {
			for &(ex, _) in self.exts.hitboxes.iter() {
				self.c.rectangle(ex.x0, ex.y0, ex.w(), ex.h());
			}
			self.c.set_source_rgb(0.0, 1.0, 0.0);
			self.c.set_line_width(1.0);
			self.c.set_line_cap(LineCap::Square);
			self.c.stroke();
		}
		
		self.c.translate(x, y);
		self.c.new_path();
		self.c.append_path(&path);
		self.c.identity_matrix();
		self.c.set_source_rgb(0.0, 0.0, 0.0);
		self.c.fill();
		
		// Draw cursor
		if self.exts.cursor_extent.is_some() {
			let ex = self.exts.cursor_extent.unwrap();
			self.c.rectangle(ex.x0, ex.y0, ex.w(), ex.h());
		}
		self.c.set_source_rgb(0.0, 0.0, 0.0);
		self.c.fill();
		
		full_extent.translate(x, y)
	}
	
	fn get_scale(&self) -> f64 {
		self.c.get_font_matrix().xx / INIT_FONT_SIZE // A bit of a hack, but whatever.
	}
	fn set_scale(&self, scale: f64) {
		self.c.set_font_size(INIT_FONT_SIZE * scale);
	}
	fn get_ascent(&self) -> f64 {
		self.c.font_extents().ascent
	}
	fn get_descent(&self) -> f64 {
		self.c.font_extents().descent / 4.0
	}

	/// Paths an expression given onto the context given. Takes into account the current position of the context and the position of the cursor given.
	/// prev_expr_extent is the extent of the token last pathed, before the current function.
	fn path_expr(&mut self, expr: VExprRef) -> Extent {
		let cursor_in_ex: bool = is_equal_reference(&expr, &self.cursor.ex);
		
		if !self.c.has_current_point() {
			self.c.move_to(0.0, 0.0);
		}
		let (current_x, current_y) = self.c.get_current_point();
		
		let mut full_extent = Extent{x0:current_x, y0:current_y, x1:current_x, y1:current_y};
		self.prev_extent.unwrap_or(self.box_extent());
		
		{ // Replace `^(x)` with `□^(x)`
			let mut i = 0;
			let mut toks = &mut expr.borrow_mut().tokens;
			loop {
				if i >= toks.len() {
					break;
				}
				if match &toks[i] { &VToken::Pow(_) if i == 0 || match &toks[i - 1] { &VToken::Char(_) | &VToken::Digit(_) => false, _ => true } => true, _ => false } {
					toks.insert(i, VToken::Space);
					i += 1;
				}
				i += 1;
			}
		}
		
		// loop through the tokens in the array
		let mut cursor_i: i64 = -1;
		for i in 0..expr.borrow().tokens.len() {
			cursor_i += 1;
			if cursor_in_ex && self.cursor.pos == cursor_i as usize {
				self.exts.cursor_extent = Some(get_cursor_extent(self.c.get_current_point(), self.get_scale()));
			}
			if cursor_in_ex && self.cursor.pos == cursor_i as usize && i != 0 && match expr.borrow().tokens[i - 1] { VToken::Space => true, _ => false } {
				self.exts.cursor_extent = None;
			}
			
			match &expr.borrow().tokens[i] {
				&VToken::Space => {
					let cursor_pos = self.cursor.pos;
					let extent = self.path_box(cursor_in_ex && cursor_pos == cursor_i as usize);
					if cursor_in_ex && self.cursor.pos == cursor_i as usize {
						self.exts.cursor_extent = None;
					}
					self.exts.push(extent, Cursor::new_ex(expr.clone(), cursor_i as usize));
					self.prev_extent = Some(extent);
					cursor_i -= 1;
				},
				&VToken::Digit(ref chr) | &VToken::Char(ref chr) => {
					let (start_x, start_y) = self.c.get_current_point();
					let s = char::to_string(&chr);
					self.c.text_path(&s);
					self.c.rel_move_to(1.0, 0.0);
					let (end_x, _) = self.c.get_current_point();
					let extent = Extent {x0:start_x, y0:start_y-self.get_ascent(), x1:end_x, y1:start_y+self.get_descent()}; // Calculate char's extent
					let (l, r)  = extent.split_lr();
					self.exts.push(l, Cursor::new_ex(expr.clone(), cursor_i as usize));
					self.exts.push(r, Cursor::new_ex(expr.clone(), cursor_i as usize + 1));
					self.prev_extent = Some(extent);
				},
				&VToken::Op(ref op) => {
					let (start_x, start_y) = self.c.get_current_point();
					let s = format!("{}", op);
					self.c.text_path(&s);
					self.c.rel_move_to(1.0, 0.0);
					let (end_x, _) = self.c.get_current_point();
					let extent = Extent {x0:start_x, y0:start_y-self.get_ascent(), x1:end_x, y1:start_y+self.get_descent()}; // Calculate char's extent
					let (l, r)  = extent.split_lr();
					self.exts.push(l, Cursor::new_ex(expr.clone(), cursor_i as usize));
					self.exts.push(r, Cursor::new_ex(expr.clone(), cursor_i as usize + 1));
					self.prev_extent = Some(extent);
				},
				&VToken::Pow(ref inner_expr) => {
					self.c.save();
					let orig_path = self.c.copy_path();
					let (orig_x, orig_y) = self.c.get_current_point();
					let orig_scale = self.get_scale();
					
					self.c.new_path();
					self.exts.push_state();
					self.set_scale(0.8);
					let base_prev_extent = self.prev_extent.unwrap();
					let mut exp_extents = self.path_expr(inner_expr.clone());
					
					let exp_path = self.c.copy_path();
					let anchor_x = base_prev_extent.x1;
					let anchor_y = base_prev_extent.y0 + base_prev_extent.h() / 2.0;
					//println!("anchor_y ({}) = prev_extent.y0 ({}) + prev_extent.h() ({}) / 2.0 ({})", anchor_y, prev_extent.y0, prev_extent.h(), prev_extent.h() / 2.0);
					let (mut x, mut y) = align(&exp_extents, anchor_x, anchor_y, TopRight);
					//self.exts.push(Extent{x0:anchor_x - 10.0, y0:anchor_y - 100.0, x1:anchor_x + 10.0, y1:anchor_y + 10.0}, Cursor::new());
					x = x.floor();
					y = y.floor();
					exp_extents = exp_extents.translate(x, y);
					exp_extents.x1 += 2.0;
					
					let descent = self.get_descent() / self.get_scale();
					self.exts.transform(|ex, is_cursor| {
						let mut new_ex = ex.translate(x, y);
						if !is_cursor {
							new_ex.y1 = orig_y + descent;
						}
						new_ex
					});
					
					{
						let mut after_extent = exp_extents;
						after_extent.x0 = after_extent.x1 - 2.0;
						after_extent.y1 = orig_y + self.get_descent() / self.get_scale();
						self.exts.push(after_extent, Cursor::new_ex(inner_expr.clone(), inner_expr.borrow().tokens.len()));
					}
					
					// All together now!
					self.c.new_path();
					self.c.append_path(&orig_path);
					self.c.translate(x, y);
					self.c.append_path(&exp_path);
					self.c.identity_matrix();
					self.c.restore();
					self.set_scale(orig_scale);
					self.c.move_to(orig_x + exp_extents.w(), orig_y); // Moves the current point onwards the width of the exp_path.
					self.prev_extent = Some(exp_extents);
				},
				&VToken::Func(FuncType::Sqrt, ref inner_expr) => {
					self.prev_extent = Some(self.path_root(inner_expr.clone(), None));
				},
				&VToken::Root(ref degree_ex, ref inner_expr) => {
					self.prev_extent = Some(self.path_root(inner_expr.clone(), Some(degree_ex.clone())));
				},
				&VToken::Func(ref func_type, ref inner_expr) => {
					// Paths the beginning of the function, the " sin("
					let (abs_orig_x, abs_orig_y) = self.c.get_current_point();
					self.c.rel_move_to(5.0, 0.0);
					self.c.text_path(format!("{}(", func_type).as_str());
					
					self.c.save();
					let orig_path = self.c.copy_path();
					let (orig_x, orig_y) = self.c.get_current_point();
					let func_ident_extent = Extent{x0:abs_orig_x, y0:abs_orig_y-self.get_ascent(), x1:orig_x, y1:orig_y+self.get_descent()};
					{
						let (l, r) = func_ident_extent.split_lr();
						self.exts.push(l, Cursor::new_ex(expr.clone(), cursor_i as usize));
						self.exts.push(r, Cursor::new_ex(inner_expr.clone(), 0));
					}
					
					self.c.new_path();
					self.exts.push_state();
					let mut inner_extents = self.path_expr(inner_expr.clone());
					
					let func_path = self.c.copy_path();
					let (mut x, _) = align(&inner_extents, orig_x, orig_y, MidRight);
					x = x.floor();
					inner_extents = inner_extents.translate(x, 0.0);
					
					self.exts.translate(x, 0.0);
					
					self.c.new_path();
					self.c.append_path(&orig_path);
					self.c.translate(x, 0.0);
					self.c.append_path(&func_path);
					self.c.restore();
					self.c.move_to(orig_x + inner_extents.w() - 1.0, orig_y); // Moves the current point onwards the width of the func_path.
					self.c.text_path(")");
					let end_x = self.c.get_current_point().0 + 1.0;
					let outer_extent = Extent{x0:orig_x, y0:orig_y-self.get_ascent(), x1:end_x, y1:orig_y+self.get_descent()};
					let func_extent = outer_extent.enclosing(&inner_extents);
					
					{
						let end_extent = Extent{x0:orig_x + inner_extents.w() - 1.0, y0:orig_y-self.get_ascent(), x1:end_x, y1:orig_y+self.get_descent()};
						let (l, r) = end_extent.split_lr();
						self.exts.push(l, Cursor::new_ex(inner_expr.clone(), inner_expr.borrow().tokens.len()));
						self.exts.push(r, Cursor::new_ex(expr.clone(), cursor_i as usize + 1));
					}
					
					self.prev_extent = Some(func_extent);
				},
				&VToken::Frac(ref num_ex, ref den_expr) => {
					self.prev_extent = Some(self.path_frac(num_ex.clone(), den_expr.clone()));
				},
			}
			full_extent = full_extent.enclosing(&self.prev_extent.unwrap_or(self.box_extent()));
		}
		
		if expr.borrow().tokens.len() != 0 {
			//self.exts.push(full_extent, Cursor::with_ex(expr.clone()));
			// Replace `□` with ``
			let mut i: i64 = 0;
			let mut toks = &mut expr.borrow_mut().tokens;
			loop {
				if i >= toks.len() as i64 {
					break;
				}
				if match toks[i as usize].clone() { VToken::Space => true, _ => false } {
					toks.remove(i as usize);
				} else {
					i += 1;
				}
			}
		}
		
		if expr.borrow().tokens.len() != 0 {
			if cursor_in_ex && self.cursor.pos == expr.borrow().tokens.len() {
				self.exts.cursor_extent = Some(get_cursor_extent(self.c.get_current_point(), self.get_scale()));
			}
		}
		
		if expr.borrow().tokens.len() == 0 {
			let cursor_pos = self.cursor.pos;
			let box_extent = self.path_box(cursor_in_ex && cursor_pos == 0);
			full_extent = full_extent.enclosing(&box_extent);
			self.c.rel_move_to(box_extent.w() - 15.0, 0.0);
			self.exts.push(box_extent, Cursor::new_ex(expr.clone(), 0));
		}
		
		full_extent
	}

	fn path_str(&mut self, s: &str) -> Extent {
		let mut ex = VExpr::new();
		for chr in s.chars() {
			if chr.is_digit(10) {
				ex.tokens.push(VToken::Digit(chr));
			} else {
				ex.tokens.push(VToken::Char(chr));
			}
		}
		if !self.c.has_current_point() {
			self.c.move_to(0.0, 0.0);
		}
		let (current_x, current_y) = self.c.get_current_point();
		
		let full_extent = Extent{x0:current_x, y0:current_y, x1:current_x, y1:current_y};
		self.prev_extent = Some(full_extent);
		self.path_expr(ex.to_ref())
	}

	fn path_root(&mut self, inner: VExprRef, degree: Option<VExprRef>) -> Extent {
		// Get the extents of the new expression
		self.c.save();
		self.exts.push_state();
		let orig_path = self.c.copy_path();
		let (orig_x, orig_y) = self.c.get_current_point();
		
		// Path inner expression, and calculate other stuff
		self.c.new_path();
		self.exts.push_state();
		let inner_extents = self.path_expr(inner.clone());
		
		let inner_path = self.c.copy_path();
		
		let inner_w = inner_extents.w();
		let inner_h = inner_extents.h();
		let inner_y_bot = inner_extents.y1;
		
		let h = inner_h + 2.0;
		let w = inner_w + 6.0;
		
		let bottom_h = (h/3.0).floor();
		//let top_h = bottom_h * 2.0;
		let scale = (h/24.0).max(1.0);
		let ground_tip_h = h/12.0;
		let ground_tip_w = 0.5*scale;
		let start_w = 8.0*scale;
		
		// Calculate starting coords for square root path
		let x = orig_x + 1.0;
		let y = inner_y_bot - bottom_h;
		
		// Align inner expression
		let (mut inner_trans_x, mut inner_trans_y) = align(&inner_extents, x + start_w - (scale - 2.0) * 3.0, inner_y_bot, TopRight);
		inner_trans_x = inner_trans_x.floor();
		inner_trans_y = inner_trans_y.floor();
		self.exts.transform(|ex, is_cursor| {
			let mut new_ex = ex.translate(inner_trans_x, inner_trans_y);
			if !is_cursor {
				new_ex.y0 = orig_y - h + 8.0;
			}
			new_ex
		});
		
		// Path degree expression.
		let (mut degree_path, mut degree_extent): (Option<_>, Option<Extent>) = (None, None);
		if degree.is_some() {
			self.c.save();
			self.c.save();
			self.c.new_path();
			self.exts.push_state();
			let scale = self.get_scale();
			self.set_scale(0.8);
			degree_extent = Some(self.path_expr(degree.unwrap().clone()));
			self.set_scale(scale);
			
			let actual_degree_path = self.c.copy_path();
			
			let (mut degree_trans_x, mut degree_trans_y) = align(&degree_extent.clone().unwrap(), x + 3.0*scale, inner_y_bot-h+15.0, TopLeft);
			degree_trans_x = degree_trans_x.floor();
			degree_trans_y = degree_trans_y.floor();
			self.exts.translate(degree_trans_x, degree_trans_y);
			
			degree_extent = Some(degree_extent.unwrap().translate(degree_trans_x, degree_trans_y));
			
			self.c.new_path();
			self.c.translate(degree_trans_x, degree_trans_y);
			self.c.append_path(&actual_degree_path);
			self.c.restore();
			degree_path = Some(self.c.copy_path());
			
			self.c.restore();
		}
		
		let mut sqrt_whole_extent = Extent{x0:orig_x, y0:inner_y_bot-h, x1:(orig_x + w + start_w).floor(), y1:inner_y_bot};
		if degree_extent.is_some() {
			sqrt_whole_extent = sqrt_whole_extent.enclosing(&degree_extent.unwrap());
		}
		let (final_align_x, _) = align(&sqrt_whole_extent, orig_x, orig_y, MidRight);
		sqrt_whole_extent = sqrt_whole_extent.translate(final_align_x, 0.0);
		self.exts.translate(final_align_x, 0.0);
		
		// 1. Path orig expression
		self.c.new_path();
		self.c.append_path(&orig_path);
		
		self.c.translate(final_align_x, 0.0);
		if degree_path.is_some() {
			self.c.append_path(&degree_path.unwrap());
		}
		
		// 2. Path square root
		self.c.move_to(x, y);
		//self.c.rel_line_to(1.5*scale, -2.0*scale);
		self.c.rel_line_to(3.0*scale, bottom_h-ground_tip_h);
		self.c.rel_line_to(3.0*scale, ground_tip_h-h);
		self.c.rel_line_to(w, 0.0);
		self.c.rel_line_to(0.0, 1.0);
		self.c.rel_line_to(1.0-w, 0.0);
		self.c.rel_line_to(-1.0-3.0*scale+ground_tip_w/2.0, h-1.0);
		self.c.rel_line_to(-ground_tip_w, 0.0);
		self.c.rel_line_to(-3.0*scale-ground_tip_w/2.0, 1.0-bottom_h);
		//self.c.rel_line_to(-2.0*scale, 2.0*scale);
		self.c.line_to(x, y);
		
		// 3. Path inner expression, using the translations
		self.c.translate(inner_trans_x, inner_trans_y);
		self.c.append_path(&inner_path);
		
		self.c.restore();
		self.c.move_to(sqrt_whole_extent.x1, orig_y);
		sqrt_whole_extent
	}

	fn path_frac(&mut self, num: VExprRef, den: VExprRef) -> Extent {
		self.c.save();
		let orig_path = self.c.copy_path();
		let (orig_x, orig_y) = self.c.get_current_point();
		let prev = self.prev_extent.unwrap_or(self.box_extent());
		let (x, y) = (orig_x, prev.y0 + prev.h()/2.0 + 5.0*self.get_scale());
		
		// Numerator
		self.c.new_path();
		let before_num = self.exts.get_state();
		let mut num_extent = self.path_expr(num.clone());
		self.exts.push(num_extent, Cursor::new_ex(num.clone(), 0));
		let after_num  = self.exts.get_state();
		let num_path = self.c.copy_path();
		
		// Denominator
		self.c.new_path();
		let before_den = self.exts.get_state();
		let mut den_extent = self.path_expr(den.clone());
		self.exts.push(den_extent, Cursor::new_ex(den.clone(), 0));
		let after_den  = self.exts.get_state();
		let den_path = self.c.copy_path();
		
		let line_w = num_extent.w().max(den_extent.w()) + 10.0;
		
		let (mut num_trans_x, mut num_trans_y) = align(&num_extent, x + line_w/2.0, y - 2.0, TopMid);
		num_trans_x = num_trans_x.floor();
		num_trans_y = num_trans_y.floor();
		num_extent = num_extent.translate(num_trans_x, num_trans_y);
		
		let (mut den_trans_x, mut den_trans_y) = align(&den_extent, x + line_w/2.0, y + 1.0, BotMid);
		den_trans_x = den_trans_x.floor();
		den_trans_y = den_trans_y.floor();
		den_extent = den_extent.translate(den_trans_x, den_trans_y);
		
		self.exts.translate_from_to(before_num, after_num, num_trans_x, num_trans_y);
		self.exts.translate_from_to(before_den, after_den, den_trans_x, den_trans_y);
		
		let line_extent = Extent{x0:x, y0:y-1.0, x1:x + line_w, y1:y};
		let full_extent = num_extent.enclosing(&den_extent).enclosing(&line_extent);
		
		
		
		self.c.new_path();
		self.c.save();
		self.c.append_path(&orig_path);
		self.c.rectangle(line_extent.x0, line_extent.y0, line_extent.w(), line_extent.h());
		self.c.translate(num_trans_x, num_trans_y);
		self.c.append_path(&num_path);
		self.c.restore();
		self.c.translate(den_trans_x, den_trans_y);
		self.c.append_path(&den_path);
		self.c.restore();
		self.c.move_to(orig_x + line_w + 2.0, orig_y);
		//self.exts.push(den_extent, Cursor::new_ex(num.clone(), 0));
		
		full_extent
	}

	fn box_extent(&mut self) -> Extent {
		let w: f64 = 14.0 * self.get_scale();
		//let h: f64 = 14.0 * self.get_scale();
		const SPACING: f64 = 1.0;
		let (x, y) = self.c.get_current_point();
		
		Extent{x0:x, y0:y-self.get_ascent(), x1:x + w + 2.0*SPACING, y1:y+self.get_descent()}
	}
	// Draws a box at the current position, with a scale that is affected by the font size.
	fn path_box(&mut self, filled: bool) -> Extent {
		let w: f64 = 14.0 * self.get_scale();
		let h: f64 = 14.0 * self.get_scale();
		const SPACING: f64 = 1.0;
		let (x, y) = self.c.get_current_point();
		
		if filled {
			// Draw a filled in box
			self.c.rectangle(x+SPACING, y, w, -h);
		} else {
			// Draw an empty box
			const INNER: f64 = 1.0; // The inner size of the empty box.
			
			self.c.rectangle(x+SPACING        , y-h    , w, INNER); //top
			self.c.rectangle(x+SPACING        , y-h    , INNER, h); //left
			self.c.rectangle(x+SPACING        , y-INNER, w, INNER); //bottom
			self.c.rectangle(x+SPACING+w-INNER, y-    h, INNER, h); // right
		}
		self.c.move_to(x + w + 2.0*SPACING, y);
		Extent{x0:x, y0:y-self.get_ascent(), x1:x + w + 2.0*SPACING, y1:y+self.get_descent()}
	}
}

#[allow(dead_code)]
#[derive(Copy, Clone, PartialEq, Eq)]
pub enum Align {
	TopLeft,
	TopMid,
	TopRight,
	MidLeft,
	Mid,
	MidRight,
	BotLeft,
	BotMid,
	BotRight,
}

/// Aligns the given path properly with the specifications given. The proper translation is given back as (x, y)
fn align(ex: &Extent, anchor_x: f64, anchor_y: f64, align: Align) -> (f64, f64) {
	let y = match align {
		TopLeft | TopMid | TopRight => anchor_y - ex.y1,
		MidLeft | Mid    | MidRight => anchor_y - (ex.y0+ex.y1)/2.0,
		BotLeft | BotMid | BotRight => anchor_y - ex.y0,
	};
	
	let x = match align {
		TopLeft  | MidLeft  | BotLeft  => anchor_x - ex.x1,
		TopMid   | Mid      | BotMid   => anchor_x - (ex.x0+ex.x1)/2.0,
		TopRight | MidRight | BotRight => anchor_x - ex.x0,
	};
	(x, y)
}

fn get_cursor_extent(pos:(f64, f64), scale:f64) -> Extent {
	Extent{ x0: pos.0 - 1.0,
		    y0: pos.1 - 24.0 * scale,
		    x1: pos.0,
		    y1: pos.1 + 6.0 * scale }
}
