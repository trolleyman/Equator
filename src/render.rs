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
	let path = c.copy_path();
	
	//let (mut x, mut y) = align(extent, alloc_w/2.0, alloc_h/2.0, Mid);
	let (mut x, mut y) = align(&extent, 15.0, 15.0, BotRight);
	x = x.floor();
	y = y.floor();
	c.translate(x, y);
	
	c.new_path();
	c.append_path(&path);
	let (x, y) = unsafe { cursor_rect_pos };
	if !x.is_nan() && !y.is_nan() {
		unsafe { c.rectangle(x - 1.0, y - 24.0 * cursor_rect_scale, 1.0, 30.0 * cursor_rect_scale) };
	}
	
	c.set_source_rgb(0.0, 0.0, 0.0);
	c.fill();
	
	//let sqrt_x = 150.0;
	//path_sqrt(c, sqrt_x, 20.0, 100.0, 28.0);
	//path_sqrt(c, sqrt_x, 60.0, 100.0, 28.0 * 2.0);
	//path_sqrt(c, sqrt_x, 120.0, 100.0, 28.0 * 3.0);
	//path_sqrt(c, sqrt_x, 190.0, 100.0, 28.0 * 4.0);
	//c.fill();
}

static mut cursor_rect_pos: (f64, f64) = (::std::f64::NAN, ::std::f64::NAN);
static mut cursor_rect_scale: f64 = 1.0;

/*fn path_sqrt(c: &Context, x:f64, y:f64, w:f64, h:f64) {
	let bottom_h = (h/3.0).floor();
	//let top_h = bottom_h * 2.0;
	let scale = (h/24.0).max(1.0);
	let ground_tip_h = h/12.0;
	let ground_tip_w = 0.5*scale;
	
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
}*/

fn path_editor(c: &Context, edit: &Editor) -> Extent {
	unsafe { cursor_rect_pos = (::std::f64::NAN, ::std::f64::NAN); cursor_rect_scale = 1.0; }
	path_expr(c, edit.root_ex.clone(), edit.ex.clone(), edit.pos)
}

fn get_scale(c: &Context) -> f64 {
	c.get_font_matrix().xx / INIT_FONT_SIZE
}

fn set_scale(c: &Context, scale: f64) {
	c.set_font_size(INIT_FONT_SIZE * scale);
}

fn get_ascent(c: &Context) -> f64 {
	
}
fn get_descent(c: &Context) -> f64 {
	
}
fn get_char_w(c: &Context, chr: char) -> f64 {
	
}

/// Paths an expression given onto the context given. Takes into account the current position of the context and the position of the cursor given.
fn path_expr(c: &Context, expr: VExprRef, cursor_expr: VExprRef, cursor_pos: usize) -> Extent {
	let cursor_in_ex: bool = unsafe {
		expr.as_unsafe_cell().get() == cursor_expr.as_unsafe_cell().get()
	};
	if !c.has_current_point() {
		c.move_to(0.0, 0.0);
	}
	
	// loop through the tokens in the array
	let len = expr.borrow().tokens.len();
	for i in 0..len {
		if cursor_in_ex && cursor_pos == i {
			unsafe { cursor_rect_pos = c.get_current_point(); cursor_rect_scale = get_scale(c); }
		}
		match &expr.borrow().tokens[i] {
			&VToken::Char(ref chr) => {
				c.text_path(char::to_string(&chr).as_str());
				c.rel_move_to(1.0, 0.0);
			},
			&VToken::Exp(ref inner_expr) => {
				c.save();
				let orig_path = c.copy_path();
				let (orig_x, orig_y) = c.get_current_point();
				let orig_scale = get_scale(c);
				
				c.new_path();
				let cursor_rect_set_before = unsafe { !cursor_rect_pos.0.is_nan() && !cursor_rect_pos.1.is_nan() };
				set_scale(c, 0.8);
				path_expr(c, inner_expr.clone(), cursor_expr.clone(), cursor_pos);
				let cursor_rect_set_after = unsafe { !cursor_rect_pos.0.is_nan() && !cursor_rect_pos.1.is_nan() };
				
				let exp_path = c.copy_path();
				let exp_extents = Extent::new(c.fill_extents());
				let (mut x, mut y) = align(&exp_extents, orig_x + 1.0, orig_y - 14.0 * get_scale(c), TopRight);
				x = x.floor();
				y = y.floor();
				
				unsafe {
					if !cursor_rect_set_before && cursor_rect_set_after {
						// Cursor_rect_pos was set in the expression. Needs to be translated.
						cursor_rect_pos.0 += x;
						cursor_rect_pos.1 += y;
					}
				}
				
				c.new_path();
				c.append_path(&orig_path);
				c.translate(x, y);
				c.append_path(&exp_path);
				c.identity_matrix();
				c.restore();
				set_scale(c, orig_scale);
				c.move_to(orig_x + exp_extents.w() + 5.0, orig_y); // Moves the current point onwards the width of the exp_path.
			},
			&VToken::Func(FuncType::Sqrt, ref inner_expr) => {
				// Get the extents of the new expression.
				c.save();
				let orig_path = c.copy_path();
				let (orig_x, orig_y) = c.get_current_point();
				
				c.new_path();
				let cursor_rect_set_before = unsafe { !cursor_rect_pos.0.is_nan() && !cursor_rect_pos.1.is_nan() };
				path_expr(c, inner_expr.clone(), cursor_expr.clone(), cursor_pos);
				let cursor_rect_set_after = unsafe { !cursor_rect_pos.0.is_nan() && !cursor_rect_pos.1.is_nan() };
				
				let inner_path = c.copy_path();
				let inner_extents = Extent::new(c.fill_extents());
				
				let inner_w = inner_extents.w();
				let inner_h = inner_extents.h();
				let inner_y_bot = inner_extents.y1;
				
				let h = inner_h + 4.0;
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
				let (mut inner_trans_x, mut inner_trans_y) = align(&inner_extents, x + start_w, inner_y_bot, TopRight);
				inner_trans_x = inner_trans_x.floor();
				inner_trans_y = inner_trans_y.floor();
				unsafe { if !cursor_rect_set_before && cursor_rect_set_after {
					// Cursor_rect_pos was set in the expression. Needs to be translated.
					cursor_rect_pos.0 += inner_trans_x;
				} }
				
				// 1. Path orig expression
				c.new_path();
				c.append_path(&orig_path);
				
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
				c.move_to(orig_x + start_w + inner_w + 10.0, orig_y);
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
				path_expr(c, inner_expr.clone(), cursor_expr.clone(), cursor_pos);
				let cursor_rect_set_after = unsafe { !cursor_rect_pos.0.is_nan() && !cursor_rect_pos.1.is_nan() };
				
				let func_path = c.copy_path();
				let func_extents = Extent::new(c.fill_extents());
				let (mut x, _) = align(&func_extents, orig_x, orig_y, MidRight);
				x = x.floor();
				
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
				c.move_to(orig_x + func_extents.w(), orig_y); // Moves the current point onwards the width of the func_path.
				c.text_path(")");
			}
		}
	}

	if len != 0 {
		if cursor_in_ex && cursor_pos == expr.borrow().tokens.len() {
			unsafe { cursor_rect_pos = c.get_current_point(); cursor_rect_scale = get_scale(c); }
		}
	}
	
	if len == 0 {
		let w: f64 = 14.0 * get_scale(c);
		let h: f64 = 14.0 * get_scale(c);
		const SPACING: f64 = 1.0;
		let (x, y) = c.get_current_point();
		
		if cursor_in_ex && cursor_pos == 0 {
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
		c.rel_move_to(w + 2.0*SPACING, 0.0);
	} else if len == 1 {
		if let VToken::Exp(_) = expr.borrow().tokens[0] {
			let w: f64 = 14.0 * get_scale(c);
			let h: f64 = 14.0 * get_scale(c);
			const SPACING: f64 = 1.0;
			let (x, y) = c.get_current_point();
			
			if cursor_in_ex && cursor_pos == 0 {
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
			c.rel_move_to(w + 2.0*SPACING, 0.0);
		}
	}
}