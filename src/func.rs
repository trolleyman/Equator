use std::fmt::{Display, Formatter};
use std::fmt::Error;

use consts::*;
use num::*;
use gui;

use self::FuncType::*;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum FuncType {
	Sqrt,
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
	Ln,
	Fact,
	Abs,
}

impl Display for FuncType {
	fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
		let s: &'static str = match self {
			&Sqrt   => "√",
			&Sin    => "sin",
			&Cos    => "cos",
			&Tan    => "tan",
			&Arsin  => "arsin",
			&Arcos  => "arcos",
			&Artan  => "artan",
			&Sinh   => "sinh",
			&Cosh   => "cosh",
			&Tanh   => "tanh",
			&Arsinh => "arsinh",
			&Arcosh => "arcosh",
			&Artanh => "artanh",
			&Ln     => "ln",
			&Fact   => "fact",
			&Abs    => "abs",
		};
		return f.write_str(s);
	}
}
impl FuncType {
	pub fn execute(&self, val: d128) -> d128 {
		let mut v = val;
		if self.is_trigonometric_in() {
			// Convert whatever is the current mode to radians
			v = match gui::get_trig_mode() {
				gui::TrigMode::Radians  => v,
				gui::TrigMode::Degrees  => v / D180 * DPI,
				gui::TrigMode::Gradians => v / D200 * DPI,
			};
		}
		
		v = match self {
			&Sqrt   => v.pow(DP5),
			&Sin    => sin(v),
			&Cos    => cos(v),
			&Tan    => tan(v),
			&Arsin  => unimplemented!(),// asin(v),
			&Arcos  => unimplemented!(),// acos(v),
			&Artan  => unimplemented!(),// atan(v),
			&Sinh   => unimplemented!(),// sinh(v),
			&Cosh   => unimplemented!(),// cosh(v),
			&Tanh   => unimplemented!(),// tanh(v),
			&Arsinh => unimplemented!(),// asinh(v),
			&Arcosh => unimplemented!(),// acosh(v),
			&Artanh => unimplemented!(),// atanh(v),
			&Ln     => unimplemented!(),// ln(v),
			&Fact   => factorial(v),
			&Abs    => v.abs(),
		};
		
		if self.is_trigonometric_out() {
			// Convert whatever is the current mode to radians
			v = match gui::get_trig_mode() {
				gui::TrigMode::Radians  => v,
				gui::TrigMode::Degrees  => v * D180 / DPI,
				gui::TrigMode::Gradians => v * D200 / DPI,
			};
		}
		
		v
	}
	
	// This function takes in radians, gives out arbritrary numbers
	fn is_trigonometric_in(&self) -> bool {
		match self {
			&Sin | &Cos | &Tan => true,
			&Arsin | &Arcos | &Artan | &Sqrt | &Sinh | &Cosh | &Tanh | &Arsinh | &Arcosh | &Artanh | &Ln | &Fact | &Abs => false,
		}
	}
	
	// This function takes in an arbritrary number, gives out radians
	fn is_trigonometric_out(&self) -> bool {
		match self {
			&Arsin | &Arcos | &Artan => true,
			&Sin | &Cos | &Tan | &Sqrt | &Sinh | &Cosh | &Tanh | &Arsinh | &Arcosh | &Artanh | &Ln | &Fact | &Abs => false,
		}
	}
}
