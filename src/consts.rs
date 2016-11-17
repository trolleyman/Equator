use num::*;

pub const CHAR_ADD: char = '+';
pub const CHAR_SUB: char = '−';
pub const CHAR_MUL: char = '×'; // ×
pub const CHAR_MUL_SIMPLE: char = '*';
pub const CHAR_DIV: char = '÷';
pub const CHAR_BOX: char = '□';
pub const CHAR_HLBOX: char = '■';

/// e
pub const E: f64 = 2.71828182845904523536028747135266249775724709369995;
/// π
pub const PI: f64 = ::std::f64::consts::PI;
/// φ
pub const GOLDEN_RATIO: f64 = 1.61803398874989484820458683436563811772030917980576286213544862270526046281890244970720720418939113748475;
/// ∞
pub const INFINITY: f64 = ::std::f64::INFINITY;
/// -∞
pub const NEG_INFINITY: f64 = ::std::f64::NEG_INFINITY;

// d128 stuff
lazy_static! {
	pub static ref D0: d128 = d128::from(0);
	pub static ref D1: d128 = d128::from(1);
	pub static ref D2: d128 = d128::from(2);
	
	pub static ref DP5: d128 = d128::from(5);
	
	pub static ref D90: d128 = d128::from(90);
	pub static ref D180: d128 = d128::from(180);
	pub static ref D200: d128 = d128::from(200);
	
	/// e
	pub static ref DE: d128 = d128!(2.71828182845904523536028747135266249775724709369995);
	/// π
	pub static ref DPI: d128 = d128!(3.14159265358979323846264338327950288419716939937510582097494459230781640628620899862803);
	/// π/2
	pub static ref DPI2: d128 = *DPI / d128::from(2);
	/// φ
	pub static ref DGOLDEN_RATIO: d128 = d128!(1.6180339887498948482045868343656381177203091798057628621354486227052604628189024497072);
	/// ∞
	pub static ref DINFINITY: d128 = d128!(Infinity);
	/// -∞
	pub static ref DNEG_INFINITY: d128 = d128!(-Infinity);
}
