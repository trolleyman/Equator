#![feature(convert, box_syntax, str_char, rc_weak, as_unsafe_cell, fmt_flags, const_fn, associated_consts)]
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
