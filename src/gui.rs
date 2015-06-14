use gtk::traits::*;
use gtk::signal::Inhibit;
use gtk::widgets::*;
use gtk::Orientation;

use gdk::{self};

use cairo::Context;

use edit::Editor;
use render::render;
use render::Extent;
use com::*;

const MAX_BUTTON_X: usize = 5;
const MAX_BUTTON_Y: usize = 3;

pub fn init_gui() {
	assert_eq!(::std::mem::size_of::<Extent>(), ::std::mem::size_of::<(f64,f64,f64,f64)>());
	
	let win: &Window = ::get_window();
	win.set_size_request(250, 350);
	
	// Get controls
	let main_grid = Grid::new().unwrap();    // This is the grid that holds all of the controls,
	main_grid.set_row_spacing(8);      // the buttons on the bottom and the drawing area on the top
	main_grid.set_column_spacing(8);
	main_grid.insert_row(0);
	main_grid.insert_column(0);
	main_grid.insert_column(1);
	main_grid.set_vexpand(true);
	main_grid.set_hexpand(true);
	
	let sep = Separator::new(Orientation::Horizontal).unwrap();
	
	let da = DrawingArea::new().unwrap();    // This is the main drawing area that the current equation is
	da.set_vexpand(true);                    // drawn to. Has a variable size.
	da.set_hexpand(true);
	//da.set_size_request(500, 500);
	da.connect_draw(|w: Widget, c: Context| {
		render(&w, &c);
		
		Inhibit(false)
	});
	
	let button_grid = get_button_grid();     // This is the 'keypad'
	
	// Connect everything
	// Need pointer to get around lifetime issue due to the fact that gtk doesn't have a lifetime.
	// Should be fine since editor exists for all of main(), and so does gtk.
	win.connect_key_press_event(|_, event| {
		let edit: &mut Editor = ::get_editor();
		let handled = edit.handle_keypress(event);
		
		let c = gdk::keyval_to_unicode(event.keyval).unwrap_or(' ');
		let name = gdk::keyval_name(event.keyval).unwrap_or(" ".to_string());
		println!("keypress: {0:#08x} : {1} : {2}", event.keyval, c, name);
		
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

pub fn dirty_expression() {
	::get_window().queue_draw();
	::get_editor().print();
	let res = match expr_to_commands(::get_editor().root_ex.clone()) {
		Ok(commands) => execute_commands(&commands),
		Err(e) => { println!("error: {}", e); return; },
	};
	match res {
		Ok(v)  => println!("result : {}", v),
		Err(e) => println!("result : error: {}", e),
	}
}

fn get_button_grid() -> Grid {
	// Get grid & size it
	let grid = Grid::new().unwrap();
	grid.set_row_spacing(3);
	grid.set_column_spacing(3);
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
		for _ in 0..MAX_BUTTON_X {
			let default_button = Button::new().unwrap();
			default_button.set_size_request(75, 23);
			default_button.set_hexpand(true);
			default_button.set_can_focus(false);
			buttons[y].push(default_button);
		}
	}
	
	// Connect each individual button
	buttons[0][0].set_label("xⁿ");
	buttons[0][1].set_label("x²");
	
	buttons[1][0].set_label("sin(x)");
	buttons[1][1].set_label("cos(x)");
	buttons[1][2].set_label("tan(x)");
	buttons[1][3].set_label("√x");
	
	buttons[2][0].set_label("∛x");
	
	buttons[0][0].connect_clicked(|_| { ::get_editor().handle_button_click(0, 0); });
	buttons[0][1].connect_clicked(|_| { ::get_editor().handle_button_click(1, 0); });
	buttons[0][2].connect_clicked(|_| { ::get_editor().handle_button_click(2, 0); });
	buttons[0][3].connect_clicked(|_| { ::get_editor().handle_button_click(3, 0); });
	buttons[0][4].connect_clicked(|_| { ::get_editor().handle_button_click(4, 0); });
	
	buttons[1][0].connect_clicked(|_| { ::get_editor().handle_button_click(0, 1); });
	buttons[1][1].connect_clicked(|_| { ::get_editor().handle_button_click(1, 1); });
	buttons[1][2].connect_clicked(|_| { ::get_editor().handle_button_click(2, 1); });
	buttons[1][3].connect_clicked(|_| { ::get_editor().handle_button_click(3, 1); });
	buttons[1][4].connect_clicked(|_| { ::get_editor().handle_button_click(4, 1); });
	
	buttons[2][0].connect_clicked(|_| { ::get_editor().handle_button_click(0, 2); });
	buttons[2][1].connect_clicked(|_| { ::get_editor().handle_button_click(1, 2); });
	buttons[2][2].connect_clicked(|_| { ::get_editor().handle_button_click(2, 2); });
	buttons[2][3].connect_clicked(|_| { ::get_editor().handle_button_click(3, 2); });
	buttons[2][4].connect_clicked(|_| { ::get_editor().handle_button_click(4, 2); });
	
	// Attach them to the grid
	for y in 0..MAX_BUTTON_Y {
		for x in 0..MAX_BUTTON_X {
			grid.attach(&buttons[y][x], x as i32, y as i32, 1, 1)
		}
	}
	
	grid // Return
}
