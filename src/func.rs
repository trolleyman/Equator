use std::fmt::{Display, Formatter};
use std::fmt::Error;

use self::FuncType::*;

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum FuncType {
	Sqrt,
	Sin,
	Cos,
	Tan,
	Arsin,
	Arcos,
	Artan,
}

impl Display for FuncType {
	fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
		let s: &'static str = match self {
			&Sqrt => "√",
			&Sin => "sin",
			&Cos => "cos",
			&Tan => "tan",
			&Arsin => "arsin",
			&Arcos => "arcos",
			&Artan => "artan",
		};
		return f.write_str(s);
	}
}
impl FuncType {
	pub fn execute(&self, v: f64) -> f64 {
		match self {
			&Sqrt => v.sqrt(),
			&Sin => v.sin(),
			&Cos => v.cos(),
			&Tan => v.tan(),
			&Arsin => v.asin(),
			&Arcos => v.acos(),
			&Artan => v.atan(),
		}
	}
}