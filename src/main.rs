#![feature(core, alloc, convert)]
#![allow(non_upper_case_globals)]
extern crate gtk;
extern crate gdk;
extern crate cairo;

use gtk::traits::*;
use gtk::{Window, WindowType, WindowPosition};
use gtk::signal::Inhibit;

use edit::Editor;

use std::mem::transmute;

pub mod vis;
pub mod edit;
pub mod func;
pub mod gui;
pub mod render;

static mut g_editor: *mut Editor = 0 as *mut Editor;
static mut g_window: *mut Window = 0 as *mut Window;

pub fn get_window() -> &'static mut Window {
	unsafe { transmute(g_window) }
}
pub fn get_editor() -> &'static mut Editor {
	unsafe { transmute(g_editor) }
}

fn main() {
	match gtk::init() {
		Err(_) => panic!("GTK could not be initialized"),
		_ => {}
	}
	
	let mut temp_edit = Editor::new();
	unsafe {
		g_editor = &mut temp_edit;
	}
	
	let mut temp_win = Window::new(WindowType::TopLevel).expect("could not create window");
	unsafe {
		g_window = &mut temp_win;
	
		let win: &Window = transmute(g_window);

		win.set_title("Equator");
		win.set_border_width(10);
		win.set_window_position(WindowPosition::Center);
		//win.set_default_size(350, 70);
		
		win.connect_delete_event(|_, _| {
			gtk::main_quit();
			Inhibit(true)
		});
	}
	
	gui::init_gui();
	
	gtk::main();
}