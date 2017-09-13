#![feature(box_syntax, const_fn)]
#![allow(non_upper_case_globals)]
extern crate gtk;
extern crate gdk;
extern crate cairo;
#[macro_use]
extern crate decimal;
#[macro_use]
extern crate lazy_static;

use gtk::prelude::*;
use gtk::{Window, WindowType, WindowPosition};

use std::ptr;

pub mod num;
pub mod vis;
pub mod edit;
pub mod func;
pub mod gui;
pub mod render;
pub mod com;
pub mod err;
pub mod consts;

static mut g_window: *mut Window = ptr::null_mut();
static mut g_editor: *mut edit::Editor = ptr::null_mut();
static mut g_vm    : *mut com::VM = ptr::null_mut();
static mut g_check_buttons: *mut gui::CheckButtons = ptr::null_mut();

pub fn get_window() -> &'static mut Window {
	unsafe {
		if g_window.is_null() {
			panic!("window not initialized");
		}
		&mut *g_window
	}
}
pub fn get_editor() -> &'static mut edit::Editor {
	unsafe {
		if g_editor.is_null() {
			panic!("editor not initialized");
		}
		&mut *g_editor
	}
}
pub fn get_vm() -> &'static mut com::VM {
	unsafe {
		if g_vm.is_null() {
			panic!("vm not initialized");
		}
		&mut *g_vm
	}
}
pub fn get_check_buttons() -> &'static mut gui::CheckButtons {
	unsafe {
		if g_check_buttons.is_null() {
			panic!("check buttons not initialized");
		}
		&mut *g_check_buttons
	}
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
	
	let mut temp_win = Window::new(WindowType::Toplevel);
	unsafe {
		g_window = &mut temp_win;
		
		let win: &Window = &temp_win;
		
		win.set_title("Equator");
		win.set_border_width(10);
		win.set_position(WindowPosition::Center);
		//win.set_default_size(350, 70);
		
		win.connect_delete_event(|_, _| {
			gtk::main_quit();
			Inhibit(true)
		});
		
		win.set_icon_from_file("icon.ico")
			.map_err(|e| println!("Warning: Could not load icon: {}", e)).ok();
	}
	
	let mut temp_vm = com::VM::new();
	unsafe {
		g_vm = &mut temp_vm;
	}
	
	let mut temp_check_buttons = gui::CheckButtons::new();
	unsafe {
		g_check_buttons = &mut temp_check_buttons;
	}
	
	gui::init_gui();
	
	gtk::main();
}
