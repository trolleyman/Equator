//#![windows_subsystem = "windows"]
#![feature(box_syntax, const_fn)]
#![allow(non_upper_case_globals)]
extern crate gtk;
extern crate gdk;
extern crate cairo;
#[macro_use]
extern crate decimal;
#[macro_use]
extern crate lazy_static;
#[cfg(windows)]
extern crate kernel32;

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

#[cfg(windows)]
fn cfg_settings() {
	let gtk_settings = gtk::Settings::get_default().expect("GTK could not be initialized");
	gtk_settings.set_property("gtk-icon-theme-name", &"Adwaita".to_value()) .expect("GTK could not be initialized");
	gtk_settings.set_property("gtk-xft-antialias"  , &1.to_value())         .expect("GTK could not be initialized");
	gtk_settings.set_property("gtk-xft-hinting"    , &1.to_value())         .expect("GTK could not be initialized");
	gtk_settings.set_property("gtk-xft-hintstyle"  , &"hintfull".to_value()).expect("GTK could not be initialized");
	gtk_settings.set_property("gtk-xft-rgba"       , &"rgb".to_value())     .expect("GTK could not be initialized");
	
	let version = unsafe { kernel32::GetVersion() as u32 };
	let major = version & 0xFF;
	let minor = version & 0xFF00;
	let build = version & 0xFFFF0000;
	
	print!("Windows version is {}.{} (build {}) ", major, minor, build);
	if major >= 10 {
		println!("(Windows 10)");
		cfg_settings_win_10();
	} else if major >= 6 && minor >= 3 {
		println!("(Windows 8.1)");
		cfg_settings_win_10();
	} else if major >= 6 && minor >= 2 {
		println!("(Windows 8)");
		cfg_settings_win_10();
	} else if major >= 6 && minor >= 1 {
		println!("(Windows 7)");
		cfg_settings_win_7();
	} else if major >= 6 {
		println!("(Windows Vista)");
		cfg_settings_win_7();
	} else if major >= 5 && minor >= 2 {
		println!("(Windows XP 64-Bit)");
		cfg_settings_win_7();
	} else if major >= 6 && minor >= 1 {
		println!("(Windows XP)");
		cfg_settings_win_7();
	} else if major >= 6 {
		println!("(Windows 2000)");
		cfg_settings_win_7();
	} else {
		println!("(Unknown)");
		cfg_settings_win_7();
	}
}

#[cfg(windows)]
fn cfg_settings_win_10() {
	// let css_provider = gtk::CssProvider::new();
	// if let Err(e) = css_provider.load_from_path("resources/win32-extra-theme/gtk.css") {
	// 	println!("Warning: Could not load win32-extra theme: {}", e);
	// 	return;
	// }
	// let screen = gdk::Screen::get_default().expect("GTK could not be initialized");
	// gtk::StyleContext::add_provider_for_screen(&screen, &css_provider, gtk::STYLE_PROVIDER_PRIORITY_APPLICATION);
	
	let gtk_settings = gtk::Settings::get_default().expect("GTK could not be initialized");
	gtk_settings.set_property("gtk-theme-name", &"win32".to_value()).expect("GTK could not be initialized");
}

#[cfg(windows)]
fn cfg_settings_win_7() {
	let gtk_settings = gtk::Settings::get_default().expect("GTK could not be initialized");
	gtk_settings.set_property("gtk-theme-name", &"win32".to_value()).expect("GTK could not be initialized");
}

#[cfg(not(windows))]
fn cfg_settings() {
	// Do nothing for now
}

fn main() {
	match gtk::init() {
		Err(_) => panic!("GTK could not be initialized"),
		_ => {}
	}
	
	cfg_settings();
	
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
		
		win.set_icon_from_file("resources/icon.ico")
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
