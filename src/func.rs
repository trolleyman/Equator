use std::fmt::{Display, Formatter};
use std::fmt::Error;

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
	pub fn execute(&self, v: f64) -> f64 {
		match self {
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