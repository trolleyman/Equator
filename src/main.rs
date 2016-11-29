#![feature(box_syntax, const_fn, associated_consts)]
#![allow(non_upper_case_globals)]
extern crate gtk;
extern crate gtk_sys;
extern crate gdk;
extern crate cairo;
#[macro_use]
extern crate decimal;
#[macro_use]
extern crate lazy_static;

#[allow(unused_imports)] // Even though this import *is* used...
use gtk::traits::*;
use gtk::{Window, WindowType, WindowPosition};
use gtk::signal::Inhibit;

use std::ffi::CString;
use std::ptr;
//use std::mem;

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
	
	let mut temp_win = Window::new(WindowType::Toplevel).expect("could not create window");
	unsafe {
		g_window = &mut temp_win;
		
		let win: &Window = &temp_win;
		
		win.set_title("Equator");
		win.set_border_width(10);
		win.set_window_position(WindowPosition::Center);
		//win.set_default_size(350, 70);
		
		win.connect_delete_event(|_, _| {
			gtk::main_quit();
			Inhibit(true)
		});
		
		gtk_sys::gtk_window_set_icon_from_file(temp_win.unwrap_widget() as *mut _, CString::new("icon.ico").unwrap().into_raw(), ptr::null_mut());
	}
	
	let mut temp_vm = com::VM::new();
	unsafe {
		g_vm = &mut temp_vm;
	}
	
	gui::init_gui();
	
	gtk::main();
}
