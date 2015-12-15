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

pub const D0: d128 = d128::from_bytes([0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 8, 34]);
pub const D1: d128 = d128::from_bytes([1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 8, 34]);
pub const D2: d128 = d128::from_bytes([2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 8, 34]);

pub const DP5: d128 = d128::from_bytes([5, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 192, 7, 34]);

pub const D90: d128 = d128::from_bytes([26, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 8, 34]);
pub const D180: d128 = d128::from_bytes([138, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 8, 34]);
pub const D200: d128 = d128::from_bytes([0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 8, 34]);

/// e
pub const DE: d128 = d128::from_bytes([98, 75, 23, 231, 90, 224, 213, 84, 68, 150, 46, 45, 132, 249, 255, 41]);
/// π
pub const DPI: d128 = d128::from_bytes([131, 230, 181, 218, 208, 98, 226, 180, 251, 179, 83, 235, 26, 204, 255, 45]);
/// π/2
pub const DPI2: d128 = d128::from_bytes([209, 231, 188, 113, 104, 49, 101, 236, 177, 246, 166, 233, 15, 239, 255, 37]);
/// φ
pub const DGOLDEN_RATIO: d128 = d128::from_bytes([56, 151, 199, 163, 186, 4, 185, 232, 97, 242, 238, 204, 128, 241, 255, 37]);
/// ∞
pub const DINFINITY: d128 = d128::from_bytes([0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 120]);
/// -∞
pub const DNEG_INFINITY: d128 = d128::from_bytes([0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 248]);
