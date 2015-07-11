#![feature(core, alloc, convert, box_syntax, str_char)]
#![allow(non_upper_case_globals)]
extern crate gtk;
extern crate gtk_sys;
extern crate gdk;
extern crate cairo;

use gtk::traits::*;
use gtk::{Window, WindowType, WindowPosition};
use gtk::signal::Inhibit;

use std::mem::transmute;

pub mod vis;
pub mod edit;
pub mod func;
pub mod gui;
pub mod render;
pub mod com;
pub mod err;
pub mod consts;
pub mod num;

static mut g_window: *mut Window = 0 as *mut Window;
static mut g_editor: *mut edit::Editor = 0 as *mut edit::Editor;
static mut g_vm    : *mut com::VM = 0 as *mut com::VM;

pub fn get_window() -> &'static mut Window {
	unsafe { transmute(g_window) }
}
pub fn get_editor() -> &'static mut edit::Editor {
	unsafe { transmute(g_editor) }
}
pub fn get_vm() -> &'static mut com::VM {
	unsafe { transmute(g_vm) }
}

fn main() {
	match gtk::init() {
		Err(_) => panic!("GTK could not be initialized"),
		_ => {}
	}
	
	let mut temp_edit = edit::Editor::new();
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
	
	let mut temp_vm = com::VM::new();
	unsafe {
		g_vm = &mut temp_vm;
	}
	
	gui::init_gui();
	
	gtk::main();
}
