use gtk::traits::*;
use gtk::signal::Inhibit;
use gtk::widgets::*;
use gtk::Orientation;

use gdk::{self};

use cairo::{Antialias, Context, FontOptions, Path};
use cairo::enums::FontSlant::*;
use cairo::enums::FontWeight::*;
use cairo::enums::HintStyle::*;
use cairo::ffi::cairo_path_extents;

use edit::Editor;
use vis::*;
use self::Align::*;

const BUTTON_W: i32 = 75;
const BUTTON_H: i32 = 23;

const MAX_BUTTON_X: usize = 5;
const MAX_BUTTON_Y: usize = 3;

const PIX_FRACTION_SPACING: f64 = 9.0;
const PIX_SPACING: f64 = 1.5;

const PIX_EXP_HEIGHT: f64 = 12.0;

const OTHER_SPACING: u32 = 8;
const BUTTON_SPACING: u32 = 3;

const FONT_SIZE: f64 = 24.0;

type Extent = (f64, f64, f64, f64);

pub fn init_gui() {
	let win: &Window = ::get_window();
	
	// Get controls
	let main_grid = Grid::new().unwrap();    // This is the grid that holds all of the controls,
	main_grid.set_row_spacing(OTHER_SPACING);      // the buttons on the bottom and the drawing area on the top
	main_grid.set_column_spacing(OTHER_SPACING);
	main_grid.insert_row(0);
	main_grid.insert_column(0);
	main_grid.insert_column(1);
	main_grid.set_vexpand(true);
	main_grid.set_hexpand(true);
	
	let sep = Separator::new(Orientation::Horizontal).unwrap();
	
	let da = DrawingArea::new().unwrap();    // This is the main drawing area that the current equation is
	da.set_vexpand(true);                    // drawn to. Has a variable size.
	da.set_hexpand(true);
	da.set_size_request(100, 100);
	da.connect_draw(|w: Widget, c: Context| {
		render(&w, &c);
		
		Inhibit(false)
	});
	
	let button_grid = get_button_grid();     // This is the 'keypad'
	
	// Connect everything
	// Need pointer to get around lifetime issue due to the fact that gtk doesn't have a lifetime.
	// Should be fine since editor exists for all of main(), and so does gtk.
	win.connect_key_press_event(|widget, event| {
		let edit: &mut Editor = ::get_editor();
		let handled = edit.handle_keypress(event);
		if handled {
			widget.get_toplevel().unwrap().queue_draw();
		}
		let c = gdk::keyval_to_unicode(event.keyval).unwrap_or(' ');
		let name = gdk::keyval_name(event.keyval).unwrap_or(" ".to_string());
		println!("key: {0:#x} : {1} : {2}", event.keyval, c, name);
		
		edit.print();
		Inhibit(handled)
	});
	
	// Add
	main_grid.attach(&da         , 0, 0, 1, 1);
	main_grid.attach(&sep        , 0, 1, 1, 1);
	main_grid.attach(&button_grid, 0, 2, 1, 1);
	
	win.add(&main_grid);
	
	// Show
	win.show_all();
}

fn get_button_grid() -> Grid {
	// Get grid & size it
	let grid = Grid::new().unwrap();
	grid.set_row_spacing(BUTTON_SPACING);
	grid.set_column_spacing(BUTTON_SPACING);
	for i in 0..MAX_BUTTON_X {
		grid.insert_column(i as i32);
	}
	for i in 0..MAX_BUTTON_Y {
		grid.insert_row(i as i32);
	}
	
	// Setup a 2D vector of buttons
	let mut buttons: Vec<Vec<Button>> = vec![];
	buttons.reserve(MAX_BUTTON_Y as usize);
	
	// Set all the buttons to a default button
	for y in 0..MAX_BUTTON_Y {
		buttons.push(vec![]);
		buttons[y].reserve(MAX_BUTTON_X as usize);
		for _x in 0..MAX_BUTTON_X {
			let default_button = Button::new().unwrap();
			default_button.set_size_request(BUTTON_W, BUTTON_H);
			default_button.set_hexpand(true);
			buttons[y].push(default_button);
		}
	}
	
	// Connect each individual button
	buttons[0][0].set_label("xⁿ");
	buttons[0][0].connect_clicked(|_| {
		let edit = ::get_editor();
		let inner_ref = VExpr::with_parent(edit.ex.clone()).to_ref();
		let exp = VToken::Exp(inner_ref.clone());
		
		edit.ex.borrow_mut().tokens.insert(edit.pos, exp);
		
		// Move cursor inside
		edit.ex = inner_ref;
		edit.pos = 0;
	});
	
	// Attach them to the grid
	for y in 0..MAX_BUTTON_Y {
		for x in 0..MAX_BUTTON_X {
			grid.attach(&buttons[y][x], x as i32, y as i32, 1, 1)
		}
	}
	
	grid // Return
}

#[allow(dead_code)]
enum Align {
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
fn align(path_extents:Extent, anchor_x:f64, anchor_y:f64, align:Align) -> (f64, f64) {
	let (x1, y1, x2, y2) = path_extents;
	let (w, h) = ((x2 - x1).abs(), (y2 - y1).abs());
	
	let y = match align {
		TopLeft | TopMid | TopRight => anchor_y - y2,
		MidLeft | Mid    | MidRight => anchor_y - (y1+y2)/2.0,
		BotLeft | BotMid | BotRight => anchor_y - y1,
	};
	
	let x = match align {
		TopLeft  | MidLeft  | BotLeft  => anchor_x - x2,
		TopMid   | Mid      | BotMid   => anchor_x - (x1+x2)/2.0,
		TopRight | MidRight | BotRight => anchor_x - x1,
	};
	
	let mut after: Extent = path_extents;
	after.0 += x; after.1 += y; after.2 += x; after.3 += y;
	
	println!("{:?} (w:{}, h:{}) Anchor:({}, {}) Transform:({}, {}) After:{:?}", path_extents, w, h, anchor_x, anchor_y, x, y, after);
	(x, y)
}

fn path_translate(c: &Context, p: &Path, trans:(f64, f64)) -> Path {
	let temp_path = c.copy_path();
	c.save();
	c.new_path();
	
	c.translate(trans.0, trans.1);
	c.append_path(p);
	let ret_path = c.copy_path();
	
	c.restore();
	c.new_path();
	c.append_path(&temp_path);
	ret_path
}

fn render(widg: &Widget, c: &Context) {
	let (alloc_w, alloc_h) = (widg.get_allocated_width() as f64, widg.get_allocated_height() as f64);
	let edit = ::get_editor();
	c.set_source_rgb(0.0, 0.0, 0.0);
	c.identity_matrix();
	c.select_font_face("CMU Serif", FontSlantNormal, FontWeightNormal);
	c.set_font_size(FONT_SIZE);
	c.set_antialias(Antialias::AntialiasBest);
	let opt = FontOptions::new();
	opt.set_antialias(Antialias::AntialiasBest);
	opt.set_hint_style(HintStyleMedium);
	c.set_font_options(opt);
	c.save();
	c.identity_matrix();
	
	let path = path_expr(c, ::get_editor().root_ex.clone());
	
	// Center the path, then draw it.
	path = path_translate(c, path, align(c.fill_extents(), alloc_w/2.0, alloc_h/2.0, Mid));
	c.append_path(&path);
	c.stroke();
	
	/*
	let e = path_extents(&c);
	c.translate(-x, -y);
	let exxx = path_extents(&c);
	println!("Actual after: {:?}", exxx);
	c.translate(x, y);
	c.fill();
	c.set_source_rgba(0.0, 0.0, 0.0, 0.2);
	c.rectangle(e.0, e.1, e.2-e.0, e.3-e.1);
	c.stroke();
	
	c.restore();
	
	c.set_source_rgba(0.0, 0.0, 0.0, 0.2);
	c.rectangle(exxx.0, exxx.1, exxx.2-exxx.0, exxx.3-exxx.1);
	c.stroke();
	
	c.set_source_rgba(0.0, 0.0, 0.0, 0.2);
	c.move_to(alloc_w/2.0, 0.0); c.line_to(alloc_w/2.0, alloc_h);
	c.move_to(0.0, alloc_h/2.0); c.line_to(alloc_w, alloc_h/2.0);
	c.stroke();*/
}

// Paths a fraction at the current point.
//  num
// -----
//  dec

fn path_fraction(c: &Context, num: VExprRef, den: VExprRef) -> Path {
	c.save();
	let (mut num_path, num_extent) = (path_expr(c, num), c.fill_extents()); // Get the numerator path & extent
	let (mut den_path, den_extent) = (path_expr(c, den), c.fill_extents()); // Get the denominator path & extent
	
	// Calculate width of fraction
	let (num_w, den_w) = (num_extent.2 - num_extent.0, den_extent.2 - den_extent.0);
	let w = num_w.max(den_w) + 2.0*PIX_FRACTION_SPACING;
	
	// Calculate alignment transformation + apply it.
	let (x_num, y_num) = align(num_extent, w/2.0, PIX_FRACTION_SPACING + 1.0, TopMid);
	c.new_path(); c.translate(x_num, y_num); c.append_path(&num_path); num_path = c.copy_path();
	let (x_den, y_den) = align(den_extent, w/2.0, PIX_FRACTION_SPACING + 1.0, BotMid);
	c.new_path(); c.translate(x_den, y_den); c.append_path(&den_path); den_path = c.copy_path();
	
	// Now draw the whole thing with the fraction line
	c.new_path(); c.move_to(0.0, 0.0);
	c.append_path(&num_path);
	c.rectangle(0.0, 0.0, w, 2.0); // Line
	c.append_path(&den_path);
	let p = c.copy_path();
	c.restore();
	p
}

fn path_expr(c: &Context, ex: VExprRef) -> Path {
	c.save();
	c.new_path(); c.move_to(0.0, 0.0);
	
	let mut expr_path: Path = c.copy_path();
	let mut s = String::with_capacity(4);
	
	let len = ex.borrow().tokens.len();
	for i in 0..len {
		match ex.borrow().tokens[i] {
			VToken::Char(chr) => {
				s.empty(); s.push(chr);
				c.text_path(s.as_str());
				expr_path = c.copy_path();
			},
			VToken::Exp(ex_ref) => {
				let current_point = c.get_current_point();
				let ex_path = path_expr(c, ex_ref); // Get the path
				let ex_extent = c.fill_extents(); // Get the extents of the path
				c.new_path(); c.save();
				c.append_path(&expr_path);
				let (trans_x, trans_y) = align(ex_extent, current_point.0 + PIX_SPACING, current_point.1 + PIX_EXP_HEIGHT, TopRight); // Redraw the expression
				c.translate(trans_x, trans_y);
				c.append_path(&ex_path);
				c.restore();
				expr_path = c.copy_path();
			},
			VToken::Func(func, ex_ref) => {
				c.text_path(func_prefix.as_str());
				let ex_path = path_expr(c, ex_ref);
				let func_prefix = format!(" {}(", func); // Get the function prefix as a string e.g. " sin("
				c.text_path(func_prefix.as_str()); // Path out the function prefix
				let (func_path, func_ext) = (c.copy_path(), c.fill_extents());
				c.append_path(&ex_path); // Then the expression's path.
				c.translate(ex_extent.3 + PIX_SPACING, 0.0);
				c.text_path(")");
				expr_path = c.copy_path();
			},
		}
		
		c.new_path(); c.move_to(0.0, 0.0);
		expr_path = c.copy_path();
	}
	
	c.restore();
	expr_path
}

//	Exp(Rc<RefCell<VExpr>>), Func(FuncType, Rc<RefCell<VExpr>>) */