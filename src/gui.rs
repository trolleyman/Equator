use gtk::prelude::*;
use gtk::{Frame, ButtonBox, Window, EventBox, Grid, RadioButton, Button, DrawingArea, CheckButton, Orientation, ReliefStyle};

use gdk;
use gdk::enums::key;

use cairo::Context;

use edit::Editor;
use render::{Render, Extent, render_result};
use com::expr_to_commands;

pub struct CheckButtons {
	shift_btn: CheckButton,
	ctrl_btn : CheckButton,
	store_btn: CheckButton,
}
impl CheckButtons {
	pub fn new() -> CheckButtons {
		CheckButtons {
			shift_btn: CheckButton::new_with_label("SHIFT"),
			ctrl_btn : CheckButton::new_with_label("CTRL" ),
			store_btn: CheckButton::new_with_label("STORE"),
		}
	}
}

#[derive(PartialEq, Eq, Copy, Clone, Debug)]
pub enum ButtonID {
	Null,
	Pow,
	Square,
	Sin,
	Cos,
	Tan,
	Arsin,
	Arcos,
	Artan,
	Sinh,
	Cosh,
	Tanh,
	Arsinh,
	Arcosh,
	Artanh,
	Sqrt,
	Cbrt,
	Frac,
	E,
	Ln,
	Fact,
	Abs,
	Var(char),
	Const(char),
}

// The expression has changed - flush the output and queue a redraw
// Also check the equation for errors...
pub fn dirty_expression() {
	println!("=== DIRTY EXPRESSION ===");
	::get_window().queue_draw();
	::get_editor().update_errors();
	::get_editor().print();
	::get_vm().clear_stack();
}

// Actually do the calculation of the current equation with the caching of the equation in history
pub fn do_calc() {
	println!("=== CALCULATING EQUATION ===");
	::get_window().queue_draw();
	::get_editor().update_errors();
	::get_editor().print();
	::get_vm().clear_stack();
	
	if ::get_editor().errors.len() != 0 {
		println!("error(s) found: {:?}", ::get_editor().errors);
	}
	
	let res = match expr_to_commands(::get_editor().root_ex.clone()) {
		Ok(commands) => ::get_vm().get_result(&commands),
		Err(e) => { println!("parse error: {}", e); return; },
	};
	match res {
		Ok(v)  => println!("result : {}", v),
		Err(e) => println!("result : error: {}", e),
	}
}

// The GUI has changed - queue a redraw buttons
pub fn dirty_gui() {
	println!("=== DIRTY GUI ===");
	::get_window().queue_draw();
	::get_editor().print();
}

pub fn init_gui() {
	assert_eq!(::std::mem::size_of::<Extent>(), ::std::mem::size_of::<(f64,f64,f64,f64)>());
	
	let win: &Window = ::get_window();
	win.set_default_size(250, 350);
	win.set_double_buffered(true);
	
	// Get controls
	let main_grid = Grid::new();    // This is the grid that holds all of the controls,
	main_grid.set_row_spacing(5);            // the buttons on the bottom and the drawing area on the top
	main_grid.set_column_spacing(5);
	main_grid.set_vexpand(true);
	main_grid.set_hexpand(true);
	
	let da_frame = Frame::new(None);
	{
		let eb = EventBox::new();
		eb.connect_button_press_event(|_, e| {
			let (x, y) = e.get_position();
			println!("mouse click: ({}, {})", x, y);
			if ::get_editor().handle_click(x, y) {
				dirty_gui();
			}
			
			Inhibit(false)
		});
		let da = DrawingArea::new();    // This is the main drawing area that the current equation is
		da.set_vexpand(true);                    // drawn to. Has a variable size.
		da.set_hexpand(true);
		//da.set_size_request(500, 500);
		da.connect_draw(|w: &DrawingArea, c: &Context| {
			let (alloc_w, alloc_h) = (w.get_allocated_width(), w.get_allocated_height());
			
			let mut ren = Render::new(&c, ::get_editor());
			ren.render(alloc_w as f64, alloc_h as f64);
			
			::get_editor().update_hitboxes(ren.exts.hitboxes.into_boxed_slice());
			
			Inhibit(false)
		});
		da.set_can_focus(true);
		da.grab_focus();
		eb.add(&da);
		da_frame.add(&eb);
	}
	
	let res_frame = Frame::new(None);
	{
		let da = DrawingArea::new();
		da.set_size_request(-1, 50);
		da.set_vexpand(false);
		da.set_hexpand(true);
		da.connect_draw(|w: &DrawingArea, c: &Context| {
			let (alloc_w, alloc_h) = (w.get_allocated_width(), w.get_allocated_height());
			
			render_result(&c, ::get_vm().get_last_result(), alloc_w as f64, alloc_h as f64);
			
			Inhibit(false)
		});
		da.set_can_focus(false);
		res_frame.add(&da);
	}
	
	let var_frame = Frame::new(Some("Variables"));
	/* Need to figure out how to remove children / iterate through them.
	let var_vbox = Box::new(Orientation::Vertical, 5);
	var_vbox.connect_draw(|w: Widget, c: Context| {
		// Add a bunch of variable controls
		let b = Box::wrap_widget(w.unwrap_widget());
		let vars: Vec<(char, Num)> = Vec::with_capacity(16);
		for (k, v) in ::get_vm().vars.iter() {
			if !com::is_constant(k) {
				vars.push((k, v));
			}
		}
		
		vars.sort();
		
		for (k, v) in vars.iter() {
			
		}
	});
	var_frame.add(var_vbox);*/
	
	let button_grid = get_button_grid();     // This is the 'keypad'
	
	// Connect everything
	// Need pointer to get around lifetime issue due to the fact that gtk doesn't have a lifetime.
	// Should be fine since editor exists for all of main(), and so does gtk.
	win.connect_key_press_event(move |_, event| {
		let edit: &mut Editor = ::get_editor();
		let handled = edit.handle_keypress(event);
		
		let c = gdk::keyval_to_unicode(event.get_keyval()).unwrap_or(' ');
		let name = gdk::keyval_name(event.get_keyval()).unwrap_or(" ".to_string());
		println!("keypress: {0:#08x} : {1} : {2}", event.get_keyval(), c, name);
		
		match event.get_keyval() {
			key::Shift_L   | key::Shift_R   => set_gui_state(GuiState::Shift),
			key::Control_L | key::Control_R => set_gui_state(GuiState::Ctrl),
			_ => {}
		}
		
		Inhibit(handled)
	});
	win.connect_key_release_event(move |_, event| {
		match event.get_keyval() {
			key::Shift_L   | key::Shift_R   => set_gui_state(GuiState::Normal),
			key::Control_L | key::Control_R => set_gui_state(GuiState::Normal),
			_ => {}
		}
		
		Inhibit(false)
	});
	
	// Add
	main_grid.attach(&da_frame   , 0, 0, 1, 1);
	main_grid.attach(&res_frame  , 0, 1, 1, 1);
	main_grid.attach(&var_frame  , 1, 0, 1, 2);
	main_grid.attach(&button_grid, 0, 2, 2, 1);
	
	win.add(&main_grid);
	//da_frame.grab_focus();
	
	// Show
	win.show_all();
}

fn get_button_grid() -> Grid {
	// Get grid & size it
	let grid = Grid::new();
	grid.set_row_spacing(3);
	grid.set_column_spacing(3);
	for i in 0..5 {
		grid.insert_column(i as i32);
	}
	for i in 0..7 {
		grid.insert_row(i as i32);
	}
	
	// Insert the radians/degrees selector
	let frame = Frame::new(None);
	{
		let rb_radians = RadioButton::new_with_mnemonic("Radians");
		rb_radians.set_focus_on_click(false);
		rb_radians.set_relief(ReliefStyle::None);
		rb_radians.connect_clicked(|but| { if but.get_active() { set_trig_mode(TrigMode::Radians); } });
		
		let rb_degrees = RadioButton::new_with_mnemonic_from_widget(&rb_radians, "Degrees");
		rb_degrees.set_focus_on_click(false);
		rb_degrees.set_relief(ReliefStyle::None);
		rb_degrees.connect_clicked(|but| { if but.get_active() { set_trig_mode(TrigMode::Degrees); } });
		
		let rb_gradians = RadioButton::new_with_mnemonic_from_widget(&rb_radians, "Gradians");
		rb_gradians.set_focus_on_click(false);
		rb_gradians.set_relief(ReliefStyle::None);
		rb_gradians.connect_clicked(|but| { if but.get_active() { set_trig_mode(TrigMode::Gradians); } });
		
		let button_box = ButtonBox::new(Orientation::Vertical); //68, 23
		button_box.add(&rb_radians);
		button_box.add(&rb_degrees);
		button_box.add(&rb_gradians);
		
		frame.add(&button_box);
	}
	grid.attach(&frame, 0, 0, 1, 3);
	
	// Setup the SHIFT + CTRL + STORE buttons.
	let shift_btn = ::get_check_buttons().shift_btn.clone();
	let ctrl_btn  = ::get_check_buttons().ctrl_btn .clone();
	let store_btn = ::get_check_buttons().store_btn.clone();
	shift_btn.set_mode(false);
	shift_btn.set_focus_on_click(false);
	ctrl_btn .set_mode(false);
	ctrl_btn .set_focus_on_click(false);
	store_btn.set_mode(false);
	store_btn.set_focus_on_click(false);
	
	shift_btn.connect_button_press_event(move |_, _| {
		// If the other is on, turn it off
		if get_gui_state() == GuiState::Shift {
			set_gui_state(GuiState::Normal);
		} else {
			set_gui_state(GuiState::Shift);
		}
		dirty_gui();
		
		Inhibit(true)
	});
	
	ctrl_btn.connect_button_press_event(move |_, _| {
		// If the other is on, turn it off
		if get_gui_state() == GuiState::Ctrl {
			set_gui_state(GuiState::Normal);
		} else {
			set_gui_state(GuiState::Ctrl);
		}
		dirty_gui();
		
		Inhibit(true)
	});
	
	store_btn.connect_button_press_event(move |_, _| {
		// If the other is on, turn it off
		if get_gui_state() == GuiState::Store {
			set_gui_state(GuiState::Normal);
		} else {
			set_gui_state(GuiState::Store);
		}
		dirty_gui();
		
		Inhibit(true)
	});
	grid.attach(&shift_btn, 1, 0, 1, 1);
	grid.attach(&ctrl_btn , 1, 1, 1, 1);
	grid.attach(&store_btn, 1, 2, 1, 1);
	
	// Connect each individual button && atttch
	make_and_attach_button(("x²", "xⁿ", ""), (ButtonID::Square, ButtonID::Pow, ButtonID::Null), &grid, 2, 0);
	make_and_attach_button(("sin", "arsin", "a"), (ButtonID::Sin, ButtonID::Arsin, ButtonID::Var('a')), &grid, 3, 0); // ⁻¹
	make_and_attach_button(("cos", "arcos", "b"), (ButtonID::Cos, ButtonID::Arcos, ButtonID::Var('b')), &grid, 4, 0);
	make_and_attach_button(("tan", "artan", "c"), (ButtonID::Tan, ButtonID::Artan, ButtonID::Var('c')), &grid, 5, 0);
	
	make_and_attach_button(("√x"  , "³√x"   , "" ), (ButtonID::Sqrt, ButtonID::Cbrt  , ButtonID::Null    ), &grid, 2, 1); // ∛
	make_and_attach_button(("sinh", "arsinh", "x"), (ButtonID::Sinh, ButtonID::Arsinh, ButtonID::Var('x')), &grid, 3, 1);
	make_and_attach_button(("cosh", "arcosh", "y"), (ButtonID::Cosh, ButtonID::Arcosh, ButtonID::Var('y')), &grid, 4, 1);
	make_and_attach_button(("tanh", "artanh", "z"), (ButtonID::Tanh, ButtonID::Artanh, ButtonID::Var('z')), &grid, 5, 1);
	
	make_and_attach_button(("π"  , "φ" , "e"), (ButtonID::Const('π'), ButtonID::Const('φ'), ButtonID::Const('e')), &grid, 2, 2);
	make_and_attach_button(("x/y", ""  , "" ), (ButtonID::Frac, ButtonID::Null, ButtonID::Null), &grid, 3, 2);
	make_and_attach_button(("eˣ" , "ln", "" ), (ButtonID::E   , ButtonID::Ln  , ButtonID::Null), &grid, 4, 2);
	make_and_attach_button(("|x|", "x!", "" ), (ButtonID::Abs , ButtonID::Fact, ButtonID::Null), &grid, 5, 2);
	
	grid // Return
}

fn make_and_attach_button(labels: (&'static str, &'static str, &'static str), ids: (ButtonID, ButtonID, ButtonID), grid: &Grid, x: i32, y: i32) {
	let but = Button::new();
	but.set_size_request(75, -1); //23
	but.set_hexpand(true);
	but.set_focus_on_click(false);
	but.set_label(labels.0);
	let ids_clone = ids.clone();
	but.connect_clicked(move |_| {
		match get_gui_state() {
			GuiState::Normal => ::get_editor().handle_button_click(ids_clone.0.clone()),
			GuiState::Shift  => ::get_editor().handle_button_click(ids_clone.1.clone()),
			GuiState::Ctrl | GuiState::Store => ::get_editor().handle_button_click(ids_clone.2.clone()),
		};
	});update_button_attrib(&but, labels.2, &ids.2);
	
	but.connect_draw(move |but, _| {
		match get_gui_state() {
			GuiState::Normal    => { update_button_attrib(&but, labels.0, &ids.0); },
			GuiState::Shift     => { update_button_attrib(&but, labels.1, &ids.1); },
			GuiState::Ctrl      => { update_button_attrib(&but, labels.2, &ids.2); },
			GuiState::Store     =>
				if let ButtonID::Var(_) = ids.2 {
					update_button_attrib(&but, labels.2, &ids.2);
				} else {
					update_button_attrib(&but, "", &ButtonID::Null);
				},
		};
		Inhibit(false)
	});
	grid.attach(&but, x, y, 1, 1);
}

// Only changes attributes passed to it when the current attributes differ
fn update_button_attrib(but: &Button, label: &'static str, id: &ButtonID) {
	let but_label = but.get_label();
	let label_string = label.to_string();
	if but_label != Some(label_string) {
		but.set_label(label);
	}
	let enabled = *id != ButtonID::Null;
	if but.get_sensitive() != enabled {
		but.set_sensitive(enabled);
	}
	/*let but_tooltip = but.get_tooltip_text();
	let tooltip: Option<_> = match id {
		&ButtonID::Var(var) | &ButtonID::Const(var) => {
			let val = ::get_editor().vm.get_var(var);
			match val {
				let var_str = format!("{}", val);
				Some(var_str)
			} else {
				None
			}
		},
		_ => None,
	};
	
	if but_tooltip != tooltip {
		if tooltip.is_some() {
			but.set_tooltip_text(tooltip.unwrap().as_str());
		} else {
			but.set_has_tooltip(false);
		}
	}*/
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GuiState {
	Normal,
	Shift,
	Ctrl,
	Store
}
static mut gui_state: GuiState = GuiState::Normal;

pub fn get_gui_state() -> GuiState {
	unsafe { gui_state.clone() }
}
pub fn set_gui_state(state: GuiState) {
	unsafe {
		let shift_btn = ::get_check_buttons().shift_btn.clone();
		let ctrl_btn  = ::get_check_buttons().ctrl_btn .clone();
		let store_btn = ::get_check_buttons().store_btn.clone();
		gui_state = state;
		match state {
			GuiState::Normal => { shift_btn.set_active(false); ctrl_btn.set_active(false); store_btn.set_active(false); },
			GuiState::Shift  => { shift_btn.set_active(true ); ctrl_btn.set_active(false); store_btn.set_active(false); },
			GuiState::Ctrl   => { shift_btn.set_active(false); ctrl_btn.set_active(true ); store_btn.set_active(false); },
			GuiState::Store  => { shift_btn.set_active(false); ctrl_btn.set_active(true ); store_btn.set_active(true ); },
		}
		dirty_gui();
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrigMode {
	Radians,
	Degrees,
	Gradians
}
static mut trig_mode: TrigMode = TrigMode::Radians;

pub fn get_trig_mode() -> TrigMode {
	unsafe { trig_mode }
}
fn set_trig_mode(new_mode: TrigMode) {
	unsafe {
		trig_mode = new_mode;
		dirty_expression();
	}
}
