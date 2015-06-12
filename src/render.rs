use gtk::traits::*;
use gtk::widgets::*;

use cairo::{Antialias, Context, FontOptions};
use cairo::enums::FontSlant::*;
use cairo::enums::FontWeight::*;
use cairo::enums::HintStyle::*;

use edit::Editor;
use vis::*;
use self::Align::*;
use func::FuncType;

#[repr(packed)]
#[derive(Debug, Clone)]
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
}

const INIT_FONT_SIZE: f64 = 24.0;

#[allow(dead_code)]
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

	let extent = path_editor(c, edit);
	//println!("{:?}", extent);
	let path = c.copy_path();
	
	//let (mut x, mut y) = align(&extent, alloc_w/2.0, alloc_h/2.0, Mid);
	let (mut x, mut y) = align(&extent, 30.0, 30.0, BotRight);
	x = x.floor();
	y = y.floor();
	c.translate(x, y);
	//extent.translate(x, y);
	
	c.new_path();
	c.append_path(&path);
	let (x, y) = unsafe { cursor_rect_pos };
	if !x.is_nan() && !y.is_nan() {
		unsafe { c.rectangle(x - 1.0, y - 24.0 * cursor_rect_scale, 1.0, 30.0 * cursor_rect_scale) };
	}
	
	c.set_source_rgb(0.0, 0.0, 0.0);
	c.fill();
	
	//c.set_source_rgb(1.0, 0.0, 0.0);
	//c.rectangle(extent.x0.floor(), extent.y0.floor(), extent.w().floor(), extent.h().floor());
	//c.set_line_width(1.0);
	//c.set_line_cap(::cairo::LineCap::LineCapSquare);
	//c.stroke();
}

static mut cursor_rect_pos: (f64, f64) = (::std::f64::NAN, ::std::f64::NAN);
static mut cursor_rect_scale: f64 = 1.0;

fn path_editor(c: &Context, edit: &Editor) -> Extent {
	unsafe { cursor_rect_pos = (::std::f64::NAN, ::std::f64::NAN); cursor_rect_scale = 1.0; }
	path_expr(c, edit.root_ex.clone(), edit.ex.clone(), edit.pos, &box_extent(c))
}

fn get_scale(c: &Context) -> f64 {
	c.get_font_matrix().xx / INIT_FONT_SIZE
}
fn set_scale(c: &Context, scale: f64) {
	c.set_font_size(INIT_FONT_SIZE * scale);
}
fn get_ascent(c: &Context) -> f64 { // TODO: Does the current text size factor into this?
	c.font_extents().ascent
}
fn get_descent(c: &Context) -> f64 {
	c.font_extents().descent / 4.0
}

/// Paths an expression given onto the context given. Takes into account the current position of the context and the position of the cursor given.
/// prev_expr_extent is the extent of the token last pathed, before the current function.
fn path_expr(c: &Context, expr: VExprRef, cursor_expr: VExprRef, cursor_pos: usize, prev_tok_extent: &Extent) -> Extent {
	let cursor_in_ex: bool = unsafe {
		expr.as_unsafe_cell().get() == cursor_expr.as_unsafe_cell().get()
	};
	if !c.has_current_point() {
		c.move_to(0.0, 0.0);
	}
	let (current_x, current_y) = c.get_current_point();
	
	let mut full_extent = Extent{x0:current_x, y0:current_y, x1:current_x, y1:current_y};
	let mut prev_extent = prev_tok_extent.clone();
	
	let len = expr.borrow().tokens.len();
	
	if len == 1 {
		if match expr.borrow().tokens[0] { VToken::Pow(_) => true, _ => false } {
			let box_extent = path_box(c, cursor_in_ex && cursor_pos == 0);
			full_extent = full_extent.enclosing(&box_extent);
		}
	}
	
	// loop through the tokens in the array
	for i in 0..len {
		if cursor_in_ex && cursor_pos == i {
			unsafe { cursor_rect_pos = c.get_current_point(); cursor_rect_scale = get_scale(c); }
		}
		match &expr.borrow().tokens[i] {
			&VToken::Char(ref chr) => {
				let (start_x, start_y) = c.get_current_point();
				let s = char::to_string(&chr);
				c.text_path(&s);
				c.rel_move_to(1.0, 0.0);
				let (end_x, _) = c.get_current_point();
				let extent = Extent {x0:start_x, y0:start_y-get_ascent(c), x1:end_x, y1:start_y+get_descent(c)}; // Calculate char's extent
				prev_extent = extent;
			},
			&VToken::Pow(ref inner_expr) => {
				c.save();
				let orig_path = c.copy_path();
				let (orig_x, orig_y) = c.get_current_point();
				let orig_scale = get_scale(c);
				
				c.new_path();
				let cursor_rect_set_before = unsafe { !cursor_rect_pos.0.is_nan() && !cursor_rect_pos.1.is_nan() };
				set_scale(c, 0.8);
				let mut exp_extents = path_expr(c, inner_expr.clone(), cursor_expr.clone(), cursor_pos, &prev_extent);
				let cursor_rect_set_after = unsafe { !cursor_rect_pos.0.is_nan() && !cursor_rect_pos.1.is_nan() };
				
				let exp_path = c.copy_path();
				let anchor_x = prev_extent.x1;
				let anchor_y = prev_extent.y0 + prev_extent.h() / 2.0;
				//println!("anchor_y ({}) = prev_extent.y0 ({}) + prev_extent.h() ({}) / 2.0 ({})", anchor_y, prev_extent.y0, prev_extent.h(), prev_extent.h() / 2.0);
				let (mut x, mut y) = align(&exp_extents, anchor_x, anchor_y, TopRight);
				x = x.floor();
				y = y.floor();
				exp_extents = exp_extents.translate(x, y);
				
				unsafe {
					if !cursor_rect_set_before && cursor_rect_set_after {
						// Cursor_rect_pos was set in the expression. Needs to be translated.
						cursor_rect_pos.0 += x;
						cursor_rect_pos.1 += y;
					}
				}
				
				// All together now!
				c.new_path();
				c.append_path(&orig_path);
				c.translate(x, y);
				c.append_path(&exp_path);
				c.identity_matrix();
				c.restore();
				set_scale(c, orig_scale);
				c.move_to(orig_x + exp_extents.w() + 2.0, orig_y); // Moves the current point onwards the width of the exp_path.
				prev_extent = exp_extents;
			},
			&VToken::Func(FuncType::Sqrt, ref inner_expr) => {
				prev_extent = path_root(c, inner_expr.clone(), None, cursor_expr.clone(), cursor_pos, &prev_extent);
			},
			&VToken::Root(ref degree_ex, ref inner_expr) => {
				prev_extent = path_root(c, inner_expr.clone(), Some(degree_ex.clone()), cursor_expr.clone(), cursor_pos, &prev_extent);
			},
			&VToken::Func(ref func_type, ref inner_expr) => {
				// Paths the beginning of the function, the " sin("
				c.rel_move_to(5.0, 0.0);
				c.text_path(format!("{}(", func_type).as_str());
				
				c.save();
				let orig_path = c.copy_path();
				let (orig_x, orig_y) = c.get_current_point();
				
				c.new_path();
				let cursor_rect_set_before = unsafe { !cursor_rect_pos.0.is_nan() && !cursor_rect_pos.1.is_nan() };
				let mut inner_extents = path_expr(c, inner_expr.clone(), cursor_expr.clone(), cursor_pos, &prev_extent);
				let cursor_rect_set_after = unsafe { !cursor_rect_pos.0.is_nan() && !cursor_rect_pos.1.is_nan() };
				
				let func_path = c.copy_path();
				let (mut x, _) = align(&inner_extents, orig_x, orig_y, MidRight);
				x = x.floor();
				inner_extents = inner_extents.translate(x, 0.0);
				
				unsafe {
					if !cursor_rect_set_before && cursor_rect_set_after {
						// Cursor_rect_pos was set in the expression. Needs to be translated.
						cursor_rect_pos.0 += x;
					}
				}
				
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
				prev_extent = func_extent;
			}
		}
		full_extent = full_extent.enclosing(&prev_extent);
	}

	if len != 0 {
		if cursor_in_ex && cursor_pos == expr.borrow().tokens.len() {
			unsafe { cursor_rect_pos = c.get_current_point(); cursor_rect_scale = get_scale(c); }
		}
	}
	
	if len == 0 {
		let box_extent = path_box(c, cursor_in_ex && cursor_pos == 0);
		full_extent = full_extent.enclosing(&box_extent);
		c.rel_move_to(box_extent.w(), 0.0);
	}
	
	full_extent
}

fn path_root(c: &Context, inner: VExprRef, degree: Option<VExprRef>, cursor_expr: VExprRef, cursor_pos: usize, prev_tok_extent: &Extent) -> Extent {
	// Get the extents of the new expression.
	let cursor_rect_set_before_root = unsafe { !cursor_rect_pos.0.is_nan() && !cursor_rect_pos.1.is_nan() };
	c.save();
	let orig_path = c.copy_path();
	let (orig_x, orig_y) = c.get_current_point();
	
	// Path inner expression, and calculate other stuff
	c.new_path();
	let cursor_rect_set_before_inner = unsafe { !cursor_rect_pos.0.is_nan() && !cursor_rect_pos.1.is_nan() };
	let inner_extents = path_expr(c, inner.clone(), cursor_expr.clone(), cursor_pos, &prev_tok_extent);
	let cursor_rect_set_after_inner = unsafe { !cursor_rect_pos.0.is_nan() && !cursor_rect_pos.1.is_nan() };
	
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
	unsafe { if !cursor_rect_set_before_inner && cursor_rect_set_after_inner {
		// Cursor_rect_pos was set in the expression. Needs to be translated.
		cursor_rect_pos.0 += inner_trans_x;
		cursor_rect_pos.1 += inner_trans_y;
	} }
	
	// Path degree expression.
	let (mut degree_path, mut degree_extent): (Option<_>, Option<Extent>) = (None, None);
	if degree.is_some() {
		c.save();
		c.save();
		c.new_path();
		let cursor_rect_set_before_degree = unsafe { !cursor_rect_pos.0.is_nan() && !cursor_rect_pos.1.is_nan() };
		let scale = get_scale(c);
		set_scale(c, 0.8);
		degree_extent = Some(path_expr(c, degree.unwrap().clone(), cursor_expr.clone(), cursor_pos, &prev_tok_extent));
		set_scale(c, scale);
		let cursor_rect_set_after_degree = unsafe { !cursor_rect_pos.0.is_nan() && !cursor_rect_pos.1.is_nan() };
		
		let actual_degree_path = c.copy_path();
		
		let (mut degree_trans_x, mut degree_trans_y) = align(&degree_extent.clone().unwrap(), x + 3.0*scale, inner_y_bot-h+15.0, TopLeft);
		degree_trans_x = degree_trans_x.floor();
		degree_trans_y = degree_trans_y.floor();
		
		degree_extent = Some(degree_extent.unwrap().translate(degree_trans_x, degree_trans_y));
		
		unsafe { if !cursor_rect_set_before_degree && cursor_rect_set_after_degree {
			// Cursor_rect_pos was set in the expression. Needs to be translated.
			cursor_rect_pos.0 += degree_trans_x;
			cursor_rect_pos.1 += degree_trans_y;
		} }
		
		c.new_path();
		c.translate(degree_trans_x, degree_trans_y);
		c.append_path(&actual_degree_path);
		c.restore();
		degree_path = Some(c.copy_path());
		
		c.restore();
	}
	let cursor_rect_set_after_root = unsafe { !cursor_rect_pos.0.is_nan() && !cursor_rect_pos.1.is_nan() };
	
	let mut sqrt_whole_extent = Extent{x0:orig_x, y0:inner_y_bot-h, x1:(orig_x + start_w + inner_w + 2.0).floor(), y1:inner_y_bot};
	if degree_extent.is_some() {
		sqrt_whole_extent = sqrt_whole_extent.enclosing(&degree_extent.unwrap());
	}
	let (final_align_x, _) = align(&sqrt_whole_extent, orig_x, orig_y, MidRight);
	sqrt_whole_extent = sqrt_whole_extent.translate(final_align_x, 0.0);
	
	unsafe { if !cursor_rect_set_before_root && cursor_rect_set_after_root {
		cursor_rect_pos.0 += final_align_x;
	} }
	
	// 1. Path orig expression
	c.new_path();
	//c.translate(final_align_x, 0.0);
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
	c.move_to(sqrt_whole_extent.x1 + 6.0, orig_y);
	sqrt_whole_extent
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
