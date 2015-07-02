use gtk::traits::*;
use gtk::widgets::*;

use cairo::{Antialias, Context, FontOptions};
use cairo::enums::FontSlant::*;
use cairo::enums::FontWeight::*;
use cairo::enums::HintStyle::*;

use edit::{Editor, Cursor};
use vis::*;
use self::Align::*;
use func::FuncType;

static mut debug_view_extents: bool = false;

#[repr(packed)]
#[derive(Debug, Clone, Copy)]
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

pub struct ExtentState {
	pub len: usize,
	pub cursor_set: bool,
}
pub struct Extents {
	pub states: Vec<ExtentState>,
	pub extents: Vec<(Extent, Cursor)>,
	pub cursor_extent: Option<Extent>
}
impl Extents {
	pub fn new() -> Extents {
		Extents { states: Vec::new(), extents: Vec::new(), cursor_extent: None }
	}
	pub fn reset(&mut self) {
		self.states.clear();
		self.extents.clear();
		self.cursor_extent = None;
	}
	pub fn get_extents_state(&self) -> ExtentState {
		ExtentState{ len:self.extents.len(), cursor_set: self.cursor_extent.is_some() }
	}
	pub fn push_extents_state(&mut self) {
		let state = self.get_extents_state();
		self.states.push(state);
	}
	pub fn push_extent(&mut self, extent: Extent, cursor: Cursor) {
		self.extents.push((extent, cursor));
	}
	pub fn translate_extents_from_to(&mut self, from: ExtentState, to: ExtentState, x: f64, y: f64) {
		if !from.cursor_set && to.cursor_set {
			self.cursor_extent = self.cursor_extent.map(|ex| ex.translate(x, y));
			// println!("translated cursor: {}, {}", x, y);
		}
		
		for i in from.len..to.len {
			self.extents[i].0 = self.extents[i].0.translate(x, y);
		}
	}
	pub fn translate_extents(&mut self, x:f64, y:f64) {
		let last_state = match self.states.pop() { Some(v) => v, _ => { println!("error: mismatching states"); return; } };
		let this_state = self.get_extents_state();
		
		self.translate_extents_from_to(last_state, this_state, x, y);
	}
	/// bool is true when it is transforming the cursor
	pub fn transform_extents_from_to<F>(&mut self, from: ExtentState, to: ExtentState, f: F) where F: Fn(Extent, bool) -> Extent {
		if !from.cursor_set && to.cursor_set {
			self.cursor_extent = self.cursor_extent.map(|ex| f(ex, true));
		}
		
		for i in from.len..to.len {
			self.extents[i].0 = f(self.extents[i].0, false);
		}
	}
	pub fn transform_extents<F>(&mut self, f: F) where F: Fn(Extent, bool) -> Extent {
		let last_state = match self.states.pop() { Some(v) => v, _ => { println!("error: mismatching states"); return; } };
		let this_state = self.get_extents_state();
		
		self.transform_extents_from_to(last_state, this_state, f);
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

#[allow(unused_variables)]
pub fn render(widg: &Widget, c: &Context) {
	let (alloc_w, alloc_h) = (widg.get_allocated_width() as f64, widg.get_allocated_height() as f64);
	let edit = ::get_editor();
	c.select_font_face("CMU Serif", FontSlantNormal, FontWeightNormal);
	c.set_font_size(INIT_FONT_SIZE);
	c.set_antialias(Antialias::AntialiasBest);
	let opt = FontOptions::new();
	opt.set_antialias(Antialias::AntialiasBest);
	opt.set_hint_style(HintStyleMedium);
	c.set_font_options(opt);
	c.identity_matrix();
	
	edit.extents = Extents::new();
	let extent = path_editor(c, edit);
	//println!("{:?}", extent);
	
	
	// Outline extent
	//c.set_source_rgb(1.0, 0.0, 0.0);
	//c.rectangle(extent.x0.floor(), extent.y0.floor(), extent.w().floor(), extent.h().floor());
	//c.set_line_width(1.0);
	//c.set_line_cap(::cairo::LineCap::LineCapSquare);
	//c.stroke();
}

fn get_cursor_extent(pos:(f64, f64), scale:f64) -> Extent {
	Extent{ x0: pos.0 - 1.0,
		    y0: pos.1 - 24.0 * scale,
		    x1: pos.0,
		    y1: pos.1 + 6.0 * scale }
}

#[allow(unused_variables)]
fn path_editor(c: &Context, edit: &mut Editor) -> Extent {
	edit.extents.push_extents_state();
	let left_extent = path_expr(c, edit.root_ex.clone(), edit.cursor.clone(), &mut edit.extents, &None);
	
	// Now path the ' = 12.512' or whatever if there is a result, else path ' = ?'
	let ret = edit.vm.get_last_result();
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
	let (x_before_eq, _) = c.get_current_point();
	let mut right_extent = path_str(c, " = ", &mut Extents::new());
	let (x_after_eq, _) = c.get_current_point();
	let x_mid = ((x_before_eq + x_after_eq) / 2.0).floor();
	right_extent = right_extent.enclosing(&path_str(c, &s, &mut Extents::new()));
	let full_extent = left_extent.enclosing(&right_extent);
	
	let path = c.copy_path();
	
	//let (mut x, mut y) = align(&full_extent, alloc_w/2.0, alloc_h/2.0, Mid); // Central
	//let (mut x, mut y) = align(&Extent{x0:x_mid, x1:x_mid, y0:full_extent.y0, y1:full_extent.y1}, alloc_w/2.0, alloc_h/2.0, Mid); // Central, with = in the middle
	let (mut x, mut y) = align(&full_extent, 30.0, 30.0, BotRight); // Eqn in top left, with 15 pix margin
	x = x.floor();
	y = y.floor();
	edit.extents.translate_extents(x, y);
	//full_extent.translate(x, y);
	
	c.identity_matrix();
	c.new_path();
	if unsafe { debug_view_extents } {
		for &(ex, _) in edit.extents.extents.iter() {
			c.rectangle(ex.x0, ex.y0, ex.w(), ex.h());
		}
		c.set_source_rgb(0.0, 1.0, 0.0);
		c.set_line_width(1.0);
		c.set_line_cap(::cairo::LineCap::LineCapSquare);
		c.stroke();
		
		for &ex in edit.highlit_extents.iter() {
			c.rectangle(ex.x0, ex.y0, ex.w(), ex.h());
		}
		c.set_source_rgb(1.0, 0.0, 0.0);
		c.set_line_width(1.0);
		c.set_line_cap(::cairo::LineCap::LineCapSquare);
		c.stroke();
	}
	
	c.translate(x, y);
	c.new_path();
	c.append_path(&path);
	c.identity_matrix();
	c.set_source_rgb(0.0, 0.0, 0.0);
	c.fill();
	
	// Draw cursor
	if edit.extents.cursor_extent.is_some() {
		let ex = edit.extents.cursor_extent.unwrap();
		c.rectangle(ex.x0, ex.y0, ex.w(), ex.h());
	}
	c.set_source_rgb(0.0, 0.0, 0.0);
	c.fill();
	
	full_extent.translate(x, y)
}

fn get_scale(c: &Context) -> f64 {
	c.get_font_matrix().xx / INIT_FONT_SIZE // A bit of a hack, but whatever.
}
fn set_scale(c: &Context, scale: f64) {
	c.set_font_size(INIT_FONT_SIZE * scale);
}
fn get_ascent(c: &Context) -> f64 {
	c.font_extents().ascent
}
fn get_descent(c: &Context) -> f64 {
	c.font_extents().descent / 4.0
}

/// Paths an expression given onto the context given. Takes into account the current position of the context and the position of the cursor given.
/// prev_expr_extent is the extent of the token last pathed, before the current function.
fn path_expr(c: &Context, expr: VExprRef, cursor: Cursor, exts: &mut Extents, prev_tok_extent: &Option<Extent>) -> Extent {
	let cursor_in_ex: bool = is_equal_reference(&expr, &cursor.ex);
	
	if !c.has_current_point() {
		c.move_to(0.0, 0.0);
	}
	let (current_x, current_y) = c.get_current_point();
	
	let mut full_extent = Extent{x0:current_x, y0:current_y, x1:current_x, y1:current_y};
	let mut prev_extent = prev_tok_extent.unwrap_or(box_extent(c));
	
	let len = expr.borrow().tokens.len();
	
	if len == 1 {
		if match expr.borrow().tokens[0] { VToken::Pow(_) => true, _ => false } {
			let box_extent = path_box(c, cursor_in_ex && cursor.pos == 0);
			full_extent = full_extent.enclosing(&box_extent);
		}
	}
	
	// loop through the tokens in the array
	for i in 0..len {
		if cursor_in_ex && cursor.pos == i {
			exts.cursor_extent = Some(get_cursor_extent(c.get_current_point(), get_scale(c)));
		}
		match &expr.borrow().tokens[i] {
			&VToken::Digit(ref chr) | &VToken::Char(ref chr) => {
				let (start_x, start_y) = c.get_current_point();
				let s = char::to_string(&chr);
				c.text_path(&s);
				c.rel_move_to(1.0, 0.0);
				let (end_x, _) = c.get_current_point();
				let extent = Extent {x0:start_x, y0:start_y-get_ascent(c), x1:end_x, y1:start_y+get_descent(c)}; // Calculate char's extent
				let (l, r)  = extent.split_lr();
				exts.push_extent(l, Cursor::new_ex(expr.clone(), i));
				exts.push_extent(r, Cursor::new_ex(expr.clone(), i + 1));
				prev_extent = extent;
			},
			&VToken::Op(ref op) => {
				let (start_x, start_y) = c.get_current_point();
				let s = format!("{}", op);
				c.text_path(&s);
				c.rel_move_to(1.0, 0.0);
				let (end_x, _) = c.get_current_point();
				let extent = Extent {x0:start_x, y0:start_y-get_ascent(c), x1:end_x, y1:start_y+get_descent(c)}; // Calculate char's extent
				let (l, r)  = extent.split_lr();
				exts.push_extent(l, Cursor::new_ex(expr.clone(), i));
				exts.push_extent(r, Cursor::new_ex(expr.clone(), i + 1));
				prev_extent = extent;
			},
			&VToken::Pow(ref inner_expr) => {
				c.save();
				let orig_path = c.copy_path();
				let (orig_x, orig_y) = c.get_current_point();
				let orig_scale = get_scale(c);
				
				c.new_path();
				exts.push_extents_state();
				set_scale(c, 0.8);
				let mut exp_extents = path_expr(c, inner_expr.clone(), cursor.clone(), exts, &None);
				
				let exp_path = c.copy_path();
				let anchor_x = prev_extent.x1;
				let anchor_y = prev_extent.y0 + prev_extent.h() / 2.0;
				//println!("anchor_y ({}) = prev_extent.y0 ({}) + prev_extent.h() ({}) / 2.0 ({})", anchor_y, prev_extent.y0, prev_extent.h(), prev_extent.h() / 2.0);
				let (mut x, mut y) = align(&exp_extents, anchor_x, anchor_y, TopRight);
				x = x.floor();
				y = y.floor();
				exp_extents = exp_extents.translate(x, y);
				exp_extents.x1 += 2.0;
				
				exts.transform_extents(|ex, is_cursor| {
					let mut new_ex = ex.translate(x, y);
					if !is_cursor {
						new_ex.y1 = orig_y + get_descent(c) / get_scale(c);
					}
					new_ex
				});
				
				{
					let mut after_extent = exp_extents;
					after_extent.x0 = after_extent.x1 - 2.0;
					after_extent.y1 = orig_y + get_descent(c) / get_scale(c);
					exts.push_extent(after_extent, Cursor::new_ex(inner_expr.clone(), inner_expr.borrow().tokens.len()));
				}
				
				// All together now!
				c.new_path();
				c.append_path(&orig_path);
				c.translate(x, y);
				c.append_path(&exp_path);
				c.identity_matrix();
				c.restore();
				set_scale(c, orig_scale);
				c.move_to(orig_x + exp_extents.w(), orig_y); // Moves the current point onwards the width of the exp_path.
				prev_extent = exp_extents;
			},
			&VToken::Func(FuncType::Sqrt, ref inner_expr) => {
				prev_extent = path_root(c, inner_expr.clone(), None, cursor.clone(), exts);
			},
			&VToken::Root(ref degree_ex, ref inner_expr) => {
				prev_extent = path_root(c, inner_expr.clone(), Some(degree_ex.clone()), cursor.clone(), exts);
			},
			&VToken::Func(ref func_type, ref inner_expr) => {
				// Paths the beginning of the function, the " sin("
				let (abs_orig_x, abs_orig_y) = c.get_current_point();
				c.rel_move_to(5.0, 0.0);
				c.text_path(format!("{}(", func_type).as_str());
				
				c.save();
				let orig_path = c.copy_path();
				let (orig_x, orig_y) = c.get_current_point();
				let func_ident_extent = Extent{x0:abs_orig_x, y0:abs_orig_y-get_ascent(c), x1:orig_x, y1:orig_y+get_descent(c)};
				{
					let (l, r) = func_ident_extent.split_lr();
					exts.push_extent(l, Cursor::new_ex(expr.clone(), i));
					exts.push_extent(r, Cursor::new_ex(inner_expr.clone(), 0));
				}
				
				c.new_path();
				exts.push_extents_state();
				let mut inner_extents = path_expr(c, inner_expr.clone(), cursor.clone(), exts, &None);
				
				let func_path = c.copy_path();
				let (mut x, _) = align(&inner_extents, orig_x, orig_y, MidRight);
				x = x.floor();
				inner_extents = inner_extents.translate(x, 0.0);
				
				exts.translate_extents(x, 0.0);
				
				c.new_path();
				c.append_path(&orig_path);
				c.translate(x, 0.0);
				c.append_path(&func_path);
				c.restore();
				c.move_to(orig_x + inner_extents.w() - 1.0, orig_y); // Moves the current point onwards the width of the func_path.
				c.text_path(")");
				let end_x = c.get_current_point().0 + 1.0;
				let outer_extent = Extent{x0:orig_x, y0:orig_y-get_ascent(c), x1:end_x, y1:orig_y+get_descent(c)};
				let func_extent = outer_extent.enclosing(&inner_extents);
				
				{
					let end_extent = Extent{x0:orig_x + inner_extents.w() - 1.0, y0:orig_y-get_ascent(c), x1:end_x, y1:orig_y+get_descent(c)};
					let (l, r) = end_extent.split_lr();
					exts.push_extent(l, Cursor::new_ex(inner_expr.clone(), inner_expr.borrow().tokens.len()));
					exts.push_extent(r, Cursor::new_ex(expr.clone(), i + 1));
				}
				
				prev_extent = func_extent;
			},
			&VToken::Frac(ref num_ex, ref den_expr) => {
				prev_extent = path_frac(c, num_ex.clone(), den_expr.clone(), cursor.clone(), exts, &prev_extent)
			},
		}
		full_extent = full_extent.enclosing(&prev_extent);
	}

	if len != 0 {
		if cursor_in_ex && cursor.pos == expr.borrow().tokens.len() {
			exts.cursor_extent = Some(get_cursor_extent(c.get_current_point(), get_scale(c)));
		}
		//exts.push_extent(full_extent, Cursor::with_ex(expr.clone()));
	}
	
	if len == 0 {
		let box_extent = path_box(c, cursor_in_ex && cursor.pos == 0);
		full_extent = full_extent.enclosing(&box_extent);
		c.rel_move_to(box_extent.w() - 15.0, 0.0);
		exts.push_extent(box_extent, Cursor::new_ex(expr.clone(), 0));
	}
	
	full_extent
}

fn path_str(c: &Context, s: &str, exts: &mut Extents) -> Extent {
	let cursor_ex = VExpr::new_ref();
	let mut ex = VExpr::new();
	for chr in s.chars() {
		if chr.is_digit(10) {
			ex.tokens.push(VToken::Digit(chr));
		} else {
			ex.tokens.push(VToken::Char(chr));
		}
	}
	if !c.has_current_point() {
		c.move_to(0.0, 0.0);
	}
	let (current_x, current_y) = c.get_current_point();
	
	let full_extent = Extent{x0:current_x, y0:current_y, x1:current_x, y1:current_y};
	path_expr(c, ex.to_ref(), Cursor::with_ex(cursor_ex), exts, &Some(full_extent))
}

fn path_root(c: &Context, inner: VExprRef, degree: Option<VExprRef>, cursor: Cursor, exts: &mut Extents) -> Extent {
	// Get the extents of the new expression
	c.save();
	exts.push_extents_state();
	let orig_path = c.copy_path();
	let (orig_x, orig_y) = c.get_current_point();
	
	// Path inner expression, and calculate other stuff
	c.new_path();
	exts.push_extents_state();
	let inner_extents = path_expr(c, inner.clone(), cursor.clone(), exts, &None);
	
	let inner_path = c.copy_path();
	
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
	exts.transform_extents(|ex, is_cursor| {
		let mut new_ex = ex.translate(inner_trans_x, inner_trans_y);
		if !is_cursor {
			new_ex.y0 = orig_y - h + 8.0;
		}
		new_ex
	});
	
	// Path degree expression.
	let (mut degree_path, mut degree_extent): (Option<_>, Option<Extent>) = (None, None);
	if degree.is_some() {
		c.save();
		c.save();
		c.new_path();
		exts.push_extents_state();
		let scale = get_scale(c);
		set_scale(c, 0.8);
		degree_extent = Some(path_expr(c, degree.unwrap().clone(), cursor.clone(), exts, &None));
		set_scale(c, scale);
		
		let actual_degree_path = c.copy_path();
		
		let (mut degree_trans_x, mut degree_trans_y) = align(&degree_extent.clone().unwrap(), x + 3.0*scale, inner_y_bot-h+15.0, TopLeft);
		degree_trans_x = degree_trans_x.floor();
		degree_trans_y = degree_trans_y.floor();
		exts.translate_extents(degree_trans_x, degree_trans_y);
		
		degree_extent = Some(degree_extent.unwrap().translate(degree_trans_x, degree_trans_y));
		
		c.new_path();
		c.translate(degree_trans_x, degree_trans_y);
		c.append_path(&actual_degree_path);
		c.restore();
		degree_path = Some(c.copy_path());
		
		c.restore();
	}
	
	let mut sqrt_whole_extent = Extent{x0:orig_x, y0:inner_y_bot-h, x1:(orig_x + w + start_w).floor(), y1:inner_y_bot};
	if degree_extent.is_some() {
		sqrt_whole_extent = sqrt_whole_extent.enclosing(&degree_extent.unwrap());
	}
	let (final_align_x, _) = align(&sqrt_whole_extent, orig_x, orig_y, MidRight);
	sqrt_whole_extent = sqrt_whole_extent.translate(final_align_x, 0.0);
	exts.translate_extents(final_align_x, 0.0);
	
	// 1. Path orig expression
	c.new_path();
	c.append_path(&orig_path);
	
	c.translate(final_align_x, 0.0);
	if degree_path.is_some() {
		c.append_path(&degree_path.unwrap());
	}
	
	// 2. Path square root
	c.move_to(x, y);
	//c.rel_line_to(1.5*scale, -2.0*scale);
	c.rel_line_to(3.0*scale, bottom_h-ground_tip_h);
	c.rel_line_to(3.0*scale, ground_tip_h-h);
	c.rel_line_to(w, 0.0);
	c.rel_line_to(0.0, 1.0);
	c.rel_line_to(1.0-w, 0.0);
	c.rel_line_to(-1.0-3.0*scale+ground_tip_w/2.0, h-1.0);
	c.rel_line_to(-ground_tip_w, 0.0);
	c.rel_line_to(-3.0*scale-ground_tip_w/2.0, 1.0-bottom_h);
	//c.rel_line_to(-2.0*scale, 2.0*scale);
	c.line_to(x, y);
	
	// 3. Path inner expression, using the translations
	c.translate(inner_trans_x, inner_trans_y);
	c.append_path(&inner_path);
	
	c.restore();
	c.move_to(sqrt_whole_extent.x1, orig_y);
	sqrt_whole_extent
}

fn path_frac(c: &Context, num: VExprRef, den: VExprRef, cursor: Cursor, exts: &mut Extents, prev_tok_extent: &Extent) -> Extent {
	c.save();
	let orig_path = c.copy_path();
	let (orig_x, orig_y) = c.get_current_point();
	let (x, y) = (orig_x, prev_tok_extent.y0 + prev_tok_extent.h()/2.0 + 5.0*get_scale(c));
	
	// Numerator
	c.new_path();
	let before_num = exts.get_extents_state();
	let mut num_extent = path_expr(c, num.clone(), cursor.clone(), exts, &None);
	exts.push_extent(num_extent, Cursor::new_ex(num.clone(), 0));
	let after_num  = exts.get_extents_state();
	let num_path = c.copy_path();
	
	// Denominator
	c.new_path();
	let before_den = exts.get_extents_state();
	let mut den_extent = path_expr(c, den.clone(), cursor, exts, &None);
	exts.push_extent(den_extent, Cursor::new_ex(den.clone(), 0));
	let after_den  = exts.get_extents_state();
	let den_path = c.copy_path();
	
	let line_w = num_extent.w().max(den_extent.w()) + 10.0;
	
	let (mut num_trans_x, mut num_trans_y) = align(&num_extent, x + line_w/2.0, y - 2.0, TopMid);
	num_trans_x = num_trans_x.floor();
	num_trans_y = num_trans_y.floor();
	num_extent = num_extent.translate(num_trans_x, num_trans_y);
	
	let (mut den_trans_x, mut den_trans_y) = align(&den_extent, x + line_w/2.0, y + 1.0, BotMid);
	den_trans_x = den_trans_x.floor();
	den_trans_y = den_trans_y.floor();
	den_extent = den_extent.translate(den_trans_x, den_trans_y);
	
	exts.translate_extents_from_to(before_num, after_num, num_trans_x, num_trans_y);
	exts.translate_extents_from_to(before_den, after_den, den_trans_x, den_trans_y);
	
	let line_extent = Extent{x0:x, y0:y-1.0, x1:x + line_w, y1:y};
	let full_extent = num_extent.enclosing(&den_extent).enclosing(&line_extent);
	
	c.new_path();
	c.save();
	c.append_path(&orig_path);
	c.rectangle(line_extent.x0, line_extent.y0, line_extent.w(), line_extent.h());
	c.translate(num_trans_x, num_trans_y);
	c.append_path(&num_path);
	c.restore();
	c.translate(den_trans_x, den_trans_y);
	c.append_path(&den_path);
	c.restore();
	c.move_to(orig_x + line_w + 2.0, orig_y);
	//exts.push_extent(den_extent, Cursor::new_ex(num.clone(), 0));
	full_extent
}

fn box_extent(c: &Context) -> Extent {
	let w: f64 = 14.0 * get_scale(c);
	//let h: f64 = 14.0 * get_scale(c);
	const SPACING: f64 = 1.0;
	let (x, y) = c.get_current_point();
	
	Extent{x0:x, y0:y-get_ascent(c), x1:x + w + 2.0*SPACING, y1:y+get_descent(c)}
}
// Draws a box at the current position, with a scale that is affected by the font size.
fn path_box(c: &Context, filled: bool) -> Extent {
	let w: f64 = 14.0 * get_scale(c);
	let h: f64 = 14.0 * get_scale(c);
	const SPACING: f64 = 1.0;
	let (x, y) = c.get_current_point();
	
	if filled {
		// Draw a filled in box
		c.rectangle(x+SPACING, y, w, -h);
	} else {
		// Draw an empty box
		const INNER: f64 = 1.0; // The inner size of the empty box.
		
		c.rectangle(x+SPACING        , y-h    , w, INNER); //top
		c.rectangle(x+SPACING        , y-h    , INNER, h); //left
		c.rectangle(x+SPACING        , y-INNER, w, INNER); //bottom
		c.rectangle(x+SPACING+w-INNER, y-    h, INNER, h); // right
	}
	c.move_to(x + w + 2.0*SPACING, y);
	Extent{x0:x, y0:y-get_ascent(c), x1:x + w + 2.0*SPACING, y1:y+get_descent(c)}
}
