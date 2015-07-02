use std::fmt::{Display, Formatter};
use std::fmt::Error;

use consts::*;
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
	pub fn execute(&self, val: f64) -> f64 {
		let mut v = val;
		if self.is_trigonometric_in() {
			// Convert whatever is the current mode to radians
			v = match gui::get_trig_mode() {
				gui::TrigMode::Radians  => v,
				gui::TrigMode::Degrees  => v / 180.0 * PI,
				gui::TrigMode::Gradians => v / 200.0 * PI,
			};
		}
		
		v = match self {
			&Sqrt   => v.sqrt(),
			&Sin    => v.sin(),
			&Cos    => v.cos(),
			&Tan    => v.tan(),
			&Arsin  => v.asin(),
			&Arcos  => v.acos(),
			&Artan  => v.atan(),
			&Sinh   => v.sinh(),
			&Cosh   => v.cosh(),
			&Tanh   => v.tanh(),
			&Arsinh => v.asinh(),
			&Arcosh => v.acosh(),
			&Artanh => v.atanh(),
			&Ln     => v.ln(),
			&Fact   => factorial(v),
			&Abs    => v.abs(),
		};
		
		if self.is_trigonometric_out() {
			// Convert whatever is the current mode to radians
			v = match gui::get_trig_mode() {
				gui::TrigMode::Radians  => v,
				gui::TrigMode::Degrees  => v * 180.0 / PI,
				gui::TrigMode::Gradians => v * 200.0 / PI,
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

/// Warning: NOT the gamma function - just floors the float and then performs some stuff.
fn factorial(f: f64) -> f64 {
	if f < 0.0 {
		::std::f64::NAN
	} else if f.floor() == 0.0 {
		1.0
	} else if !(f.floor() > 0.0) {
		::std::f64::NAN
	} else {
		factorial(f - 1.0) * f
	}	
}