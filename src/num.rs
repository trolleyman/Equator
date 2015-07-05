#[allow(unused_imports)]

use std::ops::{Add, Sub, Mul, Div, Neg, Rem};
use std::str::FromStr;
use std::cmp::{PartialOrd, Ord};
use std::fmt::{self, Write, Formatter, Display};

// Just for reference, max i64 = 9,223,372,036,854,775,807 (That's 19 digits of precision, 18 if you want a whole range).
// Number of digits of precision. So for `1.234` there is 4 digits of precision
// This cannot be higher than 16, for safety.
const PRECISION: u32 = 16;

fn times_pow_10(num: i64, exp: i32) -> i64 {
	//println!("num: {}, exp: {}", num, exp);
	if exp >= 0 {
		num * 10i64.pow(  exp  as u32)
	} else {
		if -exp > 18 {
			return 0;
		}
		num / 10i64.pow((-exp) as u32)
	}
}

fn get_digits(num: i64) -> u32 {
	for i in (0u32..19).rev() {
		if times_pow_10(num, -(i as i32)) != 0 {
			return i + 1u32;
		}
	}
	0
}

/// Base-10 number (Scientific notation)
/// `1.2345` is represented with significand = 12,345,000,000,000,000 exponent = 0
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Num {
	// Significand
	sig: i64,
	// Exponent
	exp: i16,
}
impl Num {
	pub fn new(sig_in: i64, exp: i16) -> Num {
		let mut sig = sig_in;
		let mut digits = get_digits(sig);
		while digits > PRECISION {
			sig /= 10; // Discard extra precision
			digits = get_digits(sig);
		}
		let offset_pow = PRECISION - digits;
		
		
		let new_sig = times_pow_10(sig, (offset_pow as i32));
		let new_exp = exp - offset_pow as i16;
		
		Num {sig:new_sig, exp:new_exp}
	}
	
	pub fn to_float(&self) -> f64 {
		(self.sig as f64) * 10f64.powi(self.exp as i32)
	}
}
impl Display for Num {
	fn fmt(&self, f: &mut Formatter) -> Result<(), fmt::Error> {
		let mut s = String::with_capacity(64);
		let sig = if self.sig >= 0 {
			self.sig
		} else{
			s.push('-');
			-self.sig
		};
		let before_dp = times_pow_10(sig, self.exp as i32);
		try!(write!(s, "{}.", before_dp));
		
		let mut after_dp = sig - times_pow_10(before_dp, self.exp as i32);
		
		let max_precision = f.precision().unwrap_or(10000000) as isize;
		let mut dp_printed = 0;
		let mut extra_zeros = -(PRECISION as isize + self.exp as isize);
		if extra_zeros > max_precision {
			extra_zeros = max_precision;
		}
		dp_printed += extra_zeros;
		for _ in 0..extra_zeros {
			s.push('0');
		}
		
		for i in (0i32..PRECISION as i32).rev() {
			if dp_printed > max_precision {
				break;
			}
			let digit = times_pow_10(after_dp, -i);
			
			if self.exp as i32 + i < 0 {
				s.push(match digit {
					0 => '0',
					1 => '1',
					2 => '2',
					3 => '3',
					4 => '4',
					5 => '5',
					6 => '6',
					7 => '7',
					8 => '8',
					9 => '9',
					_ => return Err(fmt::Error),
				});
			}
			after_dp -= times_pow_10(digit, i);
			dp_printed += 1;
		}
		
		let dot_pos = s.rfind('.').unwrap();
		if dot_pos == s.len() - 1 {
			let len = s.len();
			s.truncate(len - 1);
		} else {
			let non_zero_pos = s.rfind(|c| c != '0');
			if non_zero_pos.is_some() {
				if non_zero_pos.unwrap() > dot_pos {
					s.truncate(non_zero_pos.unwrap() + 1);
				} else {
					s.truncate(non_zero_pos.unwrap());
				}
			}
		}
		
		try!(f.pad_integral(sig.is_positive(), "", &s));
		Ok(())
	}
}

impl Add<Num> for Num {
	type Output = Num;
	fn add(self, rhs: Num) -> Num {
		rhs
	}
}

#[allow(dead_code)]
pub fn num_test() {
	println!("`expr` (expected:calculated)");
	test_get_digits(1);
	test_get_digits(0);
	test_get_digits(10);
	test_get_digits(001);
	test_get_digits(10218941);
	test_get_digits(19909900);
	test_get_digits(9223372036854775806);
	test_get_digits(92233720368547758);
	
	println!("");
	println!("Debug, num, float");
	test_display(Num::new(123456789, -8));
	test_display(Num::new(1001, -5));
	test_display(Num::new(-100, 0));
	test_display(Num::new(9223372036854775806, -5));
	test_display(Num::new(987654456789, -6));
}

#[allow(dead_code)]
fn test_display(num: Num) {
	println!("{:?}  \t, {: >20}, {}", num, num, num.to_float());
}

#[allow(dead_code)]
fn test_get_digits(num: i64) {
	let s = num.to_string();
	let expected_len = if num == 0 {0} else {s.len() as u32};
	let calc_len = get_digits(num);
	
	
	println!("`{}` ({}:{})", s, expected_len, calc_len);
	assert_eq!(expected_len, calc_len);
}

#[allow(dead_code)]
fn test_add(a: Num, b: Num) {
	println!("{} + {} = {}", a, b, a + b);
}
