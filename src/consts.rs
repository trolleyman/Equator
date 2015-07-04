
pub mod prelude {
	pub use super::{CHAR_ADD, CHAR_SUB, CHAR_MUL, CHAR_MUL_SIMPLE, CHAR_DIV, CHAR_BOX, CHAR_HLBOX};
	pub use super::{PI, GOLDEN_RATIO, E, INFINITY, NEG_INFINITY};
}

pub const CHAR_ADD: char = '+';
pub const CHAR_SUB: char = '−';
pub const CHAR_MUL: char = '×'; // ×
pub const CHAR_MUL_SIMPLE: char = '*';
pub const CHAR_DIV: char = '÷';
pub const CHAR_BOX: char = '□';
pub const CHAR_HLBOX: char = '■';

/// e
pub const E: f64 = ::std::f64::consts::E;
/// π
pub const PI: f64 = ::std::f64::consts::PI;
/// φ
pub const GOLDEN_RATIO: f64 = 1.61803398874989484820458683436563811772030917980576286213544862270526046281890244970720720418939113748475;
/// ∞
pub const INFINITY: f64 = ::std::f64::INFINITY;
/// -∞
pub const NEG_INFINITY: f64 = ::std::f64::NEG_INFINITY;
