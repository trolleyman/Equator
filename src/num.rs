use std::ops::{Add, Sub, Mul, Div, Neg/*, Rem*/};
use std::str::FromStr;
use std::cmp::{Ordering, PartialOrd, Ord, PartialEq, Eq};
use std::fmt::{self, Write, Formatter, Display, Debug, LowerExp, UpperExp};

// Just for reference, max i64 = 9,223,372,036,854,775,807 (That's 19 digits of precision, 18 if you want a whole range).
// Number of digits of precision. So for `1.234` there is 4 digits of precision
// This cannot be higher than 16, for safety.
const PRECISION: u32 = 16;
// This is the actual number of digits that can be held in an i64. Note: doesn't have to be full digits.
// max i64 = 9,223,372,036,854,775,807 has 19 digits. Hence, MAX_PRECISION has 19 digits
const MAX_PRECISION: u32 = 19;

const POW_TABLE: [i64; MAX_PRECISION as usize] = [
	1, 10, 100,
	1_000, 10_000, 100_000,
	1_000_000, 10_000_000, 100_000_000,
	1_000_000_000, 10_000_000_000, 100_000_000_000,
	1_000_000_000_000, 10_000_000_000_000, 100_000_000_000_000,
	1_000_000_000_000_000, 10_000_000_000_000_000, 100_000_000_000_000_000,
	1_000_000_000_000_000_000
];
#[inline(always)]
fn times_pow_10(num: i64, exp: i32) -> i64 {
	//println!("num: {}, exp: {}", num, exp);
	debug_assert!(POW_TABLE.len() == MAX_PRECISION as usize);
	debug_assert!(exp != i32::min_value());
	if exp >= 0 {
		if exp > MAX_PRECISION as i32 {
			panic!("expcannot be higher than MAX_PRECISION");
		}
		unsafe { num * POW_TABLE.get_unchecked(exp as usize) }
	} else {
		if -exp > MAX_PRECISION as i32 {
			return 0;
		}
		unsafe { num / POW_TABLE.get_unchecked((-exp) as usize) }
	}
}
#[inline(always)]
fn pow_10(exp: u32) -> i64 {
	debug_assert!(POW_TABLE.len() == MAX_PRECISION as usize);
	if exp > MAX_PRECISION as u32 {
		panic!("expcannot be higher than MAX_PRECISION");
	}
	unsafe { *POW_TABLE.get_unchecked(exp as usize) }
}
#[inline]
fn digits_len(num: i64) -> u32 {
	for i in (0u32..MAX_PRECISION).rev() {
		if num / pow_10(i) != 0 {
			return i + 1u32;
		}
	}
	0
}

/// Base-10 number (Scientific notation)
/// `1.2345` is represented with significand = 1,234,500,000,000,000 exponent = -15
#[derive(Copy, Clone)]
pub struct Num {
	// Significand
	sig: i64,
	// Exponent
	exp: i16,
}
impl Num {
	#[inline(always)]
	pub fn new(sig: i64, exp: i16) -> Num {
		Num {sig:sig, exp:exp}.simplify_sig()
	}
	
	#[inline(always)]
	pub fn zero() -> Num {
		Num{ sig:0, exp:0 }
	}
	
	pub fn simplify_sig(mut self) -> Num {
		if self.sig == 0 {
			self.exp = 0;
			return self;
		}
		
		let len = digits_len(self.sig);
		if len < PRECISION {
			let diff = PRECISION as i32 - len as i32;
			self.sig = times_pow_10(self.sig, diff);
			self.exp -= diff as i16;
		} else if len > PRECISION {
			let diff = PRECISION as i32 + 1 - len as i32; // Allow 1 last digit for rounding.
			self.sig = times_pow_10(self.sig, diff);
			self.exp -= diff as i16;
			// Round the last digit
			let mut exp_extra = 1;
			let digit = if self.sig < 0 { exp_extra = -1; -self.sig } else { self.sig } % 10;
			self.sig /= 10;
			self.exp += 1;
			if digit >= 5 {
				self.sig += exp_extra;
				self = self.simplify_sig();
			}
		}
		self
	}
	
	#[inline(always)]
	pub fn as_float(self) -> f64 {
		(self.sig as f64) * 10f64.powi(self.exp as i32)
	}
	
	#[inline(always)]
	pub fn is_positive(self) -> bool {
		self.sig >= 0
	}
	#[inline(always)]
	pub fn is_negative(self) -> bool {
		self.sig < 0
	}
	#[inline(always)]
	pub fn is_zero(self) -> bool {
		self.sig == 0
	}
	
	#[inline(always)]
	pub fn floor(self) -> Num {
		if self.exp > 0 {
			self
		} else {
			Num{sig: times_pow_10(times_pow_10(self.sig, self.exp as i32), -self.exp as i32), exp: self.exp }
		}
	}
}
impl Display for Num {
	fn fmt(&self, f: &mut Formatter) -> fmt::Result {
		let mut s = String::with_capacity(64);
		//println!("self.sig: {}, {}, {}, {}", self.sig, self.sig <= 0, self.sig == 0, self.sig >= 0);
		let (sig, is_pos) = if self.sig > 0 {
			(self.sig, true)
		} else if self.sig < 0 {
			(-self.sig, false)
		} else {
			s.push('0');
			try!(f.pad_integral(true, "", &s));
			return Ok(());
		};
		
		if self.exp < 0 {
			let before_dp: i64 = times_pow_10(sig, self.exp as i32);
			try!(write!(s, "{}.", before_dp));
		} else {
			try!(write!(s, "{}", sig));
			for _ in 0..self.exp {
				s.push('0');
			}
			s.push('.');
		}
		
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
		
		let mut sig_left: i64 = sig;
		//println!("");
		//println!("{}, {}, {}, {}", sig_left, before_dp, self.sig, self.exp);
		//println!("{: ^4}, {: ^5}, {: ^18}", "i", "digit", "sig_left");
		for i in (0i32..PRECISION as i32).rev() {
			if dp_printed > max_precision {
				break;
			}
			let digit: i64 = times_pow_10(sig_left, -i);
			//println!("{: <4}, {: <5}, {: <18}", i, digit, sig_left);
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
			sig_left -= times_pow_10(digit, i);
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
		
		try!(f.pad_integral(is_pos, "", &s));
		Ok(())
	}
}

fn fmt_num_exp(this: &Num, f: &mut Formatter, e: &str) -> fmt::Result {
	let mut s = String::with_capacity(32);
	try!(write!(s, "{}", Num::new(this.sig, -(PRECISION as i16) + 1)));
	if !this.is_positive() {
		s.remove(0);
	}
	
	s.push_str(e);
	if this.sig == 0 {
		s.push('0');
	} else {
		try!(write!(s, "{}", this.exp + PRECISION as i16 - 1));
	}
	
	//println!("s: {}, {}", s, this.is_positive());
	try!(f.pad_integral(this.is_positive(), "", &s));
	Ok(())
}
impl UpperExp for Num {
	fn fmt(&self, f: &mut Formatter) -> fmt::Result {
		fmt_num_exp(self, f, "E")
	}
}
impl LowerExp for Num {
	fn fmt(&self, f: &mut Formatter) -> fmt::Result {
		fmt_num_exp(self, f, "e")
	}
}
impl Debug for Num {
	fn fmt(&self, f: &mut Formatter) -> fmt::Result {
		write!(f, "Num {{ sig: {: >17}, exp: {: >3} }}", self.sig, self.exp)
	}
}

impl Neg for Num {
	type Output = Num;
	fn neg(mut self) -> Num {
		self.sig = -self.sig;
		self
	}
}

impl Ord for Num {
	#[inline]
	fn cmp(&self, rhs: &Num) -> Ordering {
		self.partial_cmp(rhs).unwrap()
	}
}
impl PartialOrd<Num> for Num {
	fn partial_cmp(&self, rhs: &Num) -> Option<Ordering> {
		if self.sig == 0 || rhs.sig == 0 {
			return Some(self.sig.cmp(&rhs.sig));
		}
		
		if self.exp < rhs.exp {
			return Some(Ordering::Less);
		} else if self.exp > rhs.exp {
			return Some(Ordering::Greater);
		} else {
			return Some(self.sig.cmp(&rhs.sig));
		}
	}
}

impl Eq for Num {}
impl PartialEq<Num> for Num {
	fn eq(&self, rhs: &Num) -> bool {
		if self.sig == 0 && rhs.sig == 0 {
			return true;
		} else {
			return self.sig == rhs.sig && self.exp == rhs.exp;
		}
	}
}

impl Add<Num> for Num {
	type Output = Num;
	fn add(mut self, mut rhs: Num) -> Num {
		if self.exp > rhs.exp {
			self.sig *= 10; // Increase precision by 1
			self.exp -= 1;
			self.sig += times_pow_10(rhs.sig, -(self.exp as i32 - rhs.exp as i32));
			self.simplify_sig()
		} else {
			rhs.sig *= 10;
			rhs.exp -= 1;
			rhs.sig += times_pow_10(self.sig, -(rhs.exp as i32 - self.exp as i32));
			rhs.simplify_sig()
		}
	}
}
impl Sub<Num> for Num {
	type Output = Num;
	fn sub(mut self, mut rhs: Num) -> Num {
		if self.exp > rhs.exp {
			self.sig *= 10; // Increase precision by 1
			self.exp -= 1;
			self.sig -= times_pow_10(rhs.sig, -(self.exp as i32 - rhs.exp as i32));
			self.simplify_sig()
		} else {
			rhs.sig *= 10;
			rhs.exp -= 1;
			rhs.sig -= times_pow_10(self.sig, -(rhs.exp as i32 - self.exp as i32));
			rhs.simplify_sig()
		}
	}
}

impl Mul<Num> for Num {
	type Output = Num;
	fn mul(mut self, mut rhs: Num) -> Num {
		if self.is_zero() || rhs.is_zero() {
			return Num::zero();
		}
		
		//println!("{:?} * {:?}", self, rhs);
		
		let offset = (PRECISION as i32 / 2) - 2;
		let ret_exp = self.exp + rhs.exp + 2*offset as i16;
		self.sig = times_pow_10(self.sig, -offset);
		rhs.sig  = times_pow_10(rhs.sig , -offset);
		//println!("{:?} * {:?}", self, rhs);
		let ret_sig = self.sig * rhs.sig;
		Num::new(ret_sig, ret_exp).simplify_sig()
		//println!(" = {:?}", ret);
		
	}
}
impl Div<Num> for Num {
	type Output = Num;
	fn div(self, rhs: Num) -> Num {
		if rhs.is_zero() {
			panic!("divided by zero");
		} else if self.is_zero() {
			return Num::zero();
		}
		
		// Long division
		//let negative_ret: bool = (self.is_negative()) ^ (rhs.is_negative());
		let ret_exp = self.exp - rhs.exp - PRECISION as i16 - 2;
		
		let mut result = 0;
		let mut a = self.sig * 100; // eqn = a / b
		let mut b = rhs.sig * 100;
		
		//println!("a: {: >20}, b: {: >20}, result: {:?}", a, b, result);
		let mut i = PRECISION + 2;
		while a != 0 && b != 0 {
			let mul = a / b;
			result = result + mul * pow_10(i);
			a -= b * mul;
			b /= 10;
			i -= 1;
			//println!("mul: {: >20}, a: {: >20}, b: {: >20}, result: {:?}", mul, a, b, result);
		}
		
		Num{sig:result, exp:ret_exp}.simplify_sig()
		
		/* // Non-long division - low precision
		let ret_exp = self.exp - rhs.exp - PRECISION as i16 - 1;
		//let rem = (self.sig % rhs.sig);
		let ret_sig = ((self.sig * 1000) / (rhs.sig / 100000000000000));// + rem;
		
		println!("ret: {:?}", Num{sig:ret_sig, exp:ret_exp});
		Num{sig:ret_sig, exp:ret_exp}.simplify_sig()*/
	}
}

pub struct NumParseError;
impl FromStr for Num {
	type Err = NumParseError;
	fn from_str(s: &str) -> Result<Num, NumParseError> {
		// Assert that all of the characters in the str are digits or full stops, or es, and set the proper variables.
		if s.len() == 0 {
			return Err(NumParseError);
		}
		let (mut dp_pos, mut e_pos) = (None, None);
		for (i, c) in s.char_indices() {
			if c.is_digit(10) {
				// Do nothing
			} else if c == '-' {
				if i != 0 && ((!e_pos.is_some()) && i != e_pos.unwrap() + 1) {
					return Err(NumParseError);
				}
			} else if c == '.' {
				if dp_pos.is_some() {
					return Err(NumParseError);
				}
				dp_pos = Some(i);
			} else if c == 'e' || c == 'E' {
				if e_pos.is_some() {
					return Err(NumParseError);
				}
				e_pos = Some(i);
			} else {
				return Err(NumParseError);
			}
		}
		
		// Now set the strs for each section
		//println!("s: `{}`, dp_pos: {:?}, e_pos: {:?}", s, dp_pos, e_pos);
		let negative = s.char_at(0) == '-';
		let start_pos = if negative { 1 } else { 0 };
		let mid_pos = dp_pos.unwrap_or(e_pos.unwrap_or(s.len()));
		let end_pos = e_pos.unwrap_or(s.len());
		//println!("s: `{}`, {}..{}..{}", s, start_pos, mid_pos, end_pos);
		
		let mut before_dp = if mid_pos == start_pos { "" } else { &s[start_pos..mid_pos] };
		//println!("1");
		let after_dp = if mid_pos == s.len() || mid_pos >= end_pos { "" } else { &s[mid_pos + 1..end_pos] };
		//println!("2");
		let exp_str = if end_pos == s.len() || end_pos >= s.len() { "0" } else { &s[end_pos + 1..s.len()] };
		//println!("3");
		
		//println!("s: `{}` | `{}` | `{}` | `{}`", s, before_dp, after_dp, exp_str);
		
		if (before_dp.is_empty() && after_dp.is_empty() && exp_str.is_empty()) // '.e'
			|| (exp_str.is_empty() && e_pos.is_some()) // '1.4e' (exp required when e is used)
				{
			return Err(NumParseError);
		}
		
		// Concatenate the before the dp and after the dp for convenience.
		before_dp = before_dp.trim_left_matches('0');
		let ret_exp: i16 = if before_dp.len() == 0 {
			-((after_dp.len() - after_dp.trim_left_matches('0').len()) as i16)
		} else {
			before_dp.len() as i16
		};
		let mut cur_pow = PRECISION + 1;
		let mut ret_sig: i64 = 0;
		for c in before_dp.chars().chain(after_dp.chars()).skip_while(|c| *c == '0') {
			ret_sig += match c.to_digit(10) { Some(v) => v as i64, None => return Err(NumParseError) } * pow_10(cur_pow);
			//println!("{} : {: >20}", c, ret_sig);
			
			if cur_pow == 0 {
				break;
			}
			cur_pow -= 1;
		}
		let extra_exp: i16 = match exp_str.parse::<i16>() {
			Ok(e) => e,
			Err(_) => return Err(NumParseError)
		};
		// 1.4200   1420000000000000 -15
		// 000.0024 2400000000000000 -18
		
		let num = Num{sig:ret_sig, exp:ret_exp + extra_exp - PRECISION as i16 - 2}.simplify_sig();
		if negative {
			Ok(-num)
		} else {
			Ok( num)
		}
	}
}

#[allow(dead_code)]
pub fn num_test() {
	
}

#[test]
fn test_digits_len() {
	fn test_one(num: i64) {
		let s = num.to_string();
		let expected_len = if num == 0 {0} else {s.len() as u32};
		let calc_len = digits_len(num);
		
		println!("`{}` ({}:{})", s, expected_len, calc_len);
		assert_eq!(expected_len, calc_len);
	}
	
	println!(" == Testing Num Len == ");
	println!("`expr` (expected:calculated)");
	test_one(1);
	test_one(0);
	test_one(10);
	test_one(001);
	test_one(10218941);
	test_one(19909900);
	test_one(9223372036854775806);
	test_one(92233720368547758);
}
#[test]
fn test_display() {
	fn test_one(num: Num, expected_exp: &str, expected_display: &str) {
		let calc_display = format!("{:}", num);
		let calc_exp = format!("{:e}", num);
		println!("{:?}, {: >22}, {: >20}, {}", num, calc_exp, calc_display, num.as_float());
		assert_eq!(calc_display, expected_display);
		assert_eq!(calc_exp, expected_exp);
	}
	
	println!(" == Testing Display == ");
	println!("{: ^40}, {: ^22}, {: ^20}, {: ^15}", "Debug", "UpperExp", "num", "float");
	test_one(Num::new(0, 0), "0e0", "0");
	test_one(Num::new(123456789, -20), "1.23456789e-12", "0.00000000000123456789");
	test_one(Num::new(123456789, -8), "1.23456789e0", "1.23456789");
	test_one(Num::new(1001001001001, -5), "1.001001001001e7", "10010010.01001");
	test_one(Num::new(-100, 0), "-1e2", "-100");
	test_one(Num::new(9999999999999999  , 0), "9.999999999999999e15", "9999999999999999"); // 16 9s
	test_one(Num::new(99999999999999989 , 0), "9.999999999999999e16", "99999999999999990"); // 16 9s, 1 8
	test_one(Num::new(99999999999999999 , 0), "1e17", "100000000000000000"); // 17 9s
	test_one(Num::new(-9999999999999999 , 0), "-9.999999999999999e15", "-9999999999999999"); // 16 9s
	test_one(Num::new(-99999999999999989, 0), "-9.999999999999999e16", "-99999999999999990"); // 16 9s, 1 8
	test_one(Num::new(-99999999999999999, 0), "-1e17", "-100000000000000000"); // 17 9s
	test_one(Num::new(922337203685477580 , 0), "9.223372036854776e17", "922337203685477600"); // Demonstration that extra precision is discarded.
	test_one(Num::new(9223372036854775806, 0), "9.223372036854776e18", "9223372036854776000");
	test_one(Num::new(123456789, 45), "1.23456789e53", "123456789000000000000000000000000000000000000000000000");
	test_one(Num::new(987654456789, -6), "9.87654456789e5", "987654.456789");
}
#[test]
fn test_cmp() {
	fn test_one(a: Num, b: Num, expected: Ordering) {
		let cmp_s = match expected {
			Ordering::Less     => "<",
			Ordering::Equal    => "=",
			Ordering::Greater  => ">",
		};
		let calc = a.cmp(&b);
		println!("{:?} {} {:?} <=> {: >20} {} {: <20} ? {}", a, cmp_s, b, a, cmp_s, b, calc == expected);
		assert_eq!(expected, calc);
	}
	
	println!(" == Testing Comparisons == ");
	test_one(Num::new(0, 0), Num::new(0,  10), Ordering::Equal);
	test_one(Num::new(0, 0), Num::new(0, -10), Ordering::Equal);
	test_one(Num::new(10, 0), Num::new(0, 10), Ordering::Greater);
	test_one(Num::new(0, 0), Num::new(10, 0), Ordering::Less);
	test_one(Num::new(1000, 0), Num::new(1000, 1), Ordering::Less);
	test_one(Num::new(1001, 5), Num::new(1000, 5), Ordering::Greater);
	test_one(Num::new(123456789, 5), Num::new(123456789, -5), Ordering::Greater);
	test_one(Num::new(10, 0), Num::new(-10, 0), Ordering::Greater);
	test_one(Num::new(-15, 0), Num::new(-10, 0), Ordering::Less);
	test_one(Num::new(-15, 3), Num::new(-10, 0), Ordering::Greater);
	test_one(Num::new(10, 4), Num::new(1, 5), Ordering::Equal);
	test_one(Num::new(123_000, 0), Num::new(123, 3), Ordering::Equal);
}

#[test]
fn test_add() {
	fn test_one(a: Num, b: Num, expected: Num) {
		let calc1 = a + b;
		let calc2 = b + a;
		println!("{} + {} = {} ({}) ? {}", a, b, calc1, expected, calc1 == expected);
		assert_eq!(calc1, expected);
		println!("{} + {} = {} ({}) ? {}", b, a, calc2, expected, calc2 == expected);
		assert_eq!(calc2, expected);
	}
	
	println!(" == Testing Addition == ");
	test_one(Num::new(100, 0), Num::new(-100, 0), Num::new(0, 0));
	test_one(Num::new(12345, 3), Num::new(12345, 0), Num::new(12357345, 0));
	test_one(Num::new(999888777666555, 0), Num::new(999888777666555, 0), Num::new(1999777555333110, 0));
	test_one(Num::new(9998887776665558, 0), Num::new(9998887776665558, 0), Num::new(19997775553331120, 0)); // Actual answer is 19997775553331116, but Num rounds the result.
	test_one(Num::new(9998887776665558, 5), Num::new(9998887776665558, -3), Num::new(9998887876654436, 5)); // Actual answer is 999888787665443576665.558, but Num rounds the result.
	test_one(Num::new(-9998887776665558, 5), Num::new(-9998887776665558, -3), Num::new(-9998887876654436, 5)); // Actual answer is -999888787665443576665.558, but Num rounds the result.
}
#[test]
fn test_sub() {
	fn test_one(a: Num, b: Num, expected: Num, opt_expected2: Option<Num>) {
		let calc1 = a - b;
		let calc2 = b - a;
		println!("{} - {} = {} ({}) ? {}", a, b, calc1, expected, calc1 == expected);
		assert_eq!(calc1, expected);
		let expected2 = opt_expected2.unwrap_or(-expected);
		println!("{} - {} = {} ({}) ? {}", b, a, calc2, expected2, calc2 == expected2);
		assert_eq!(calc2, expected2);
	}
	
	println!(" == Testing Subtraction == ");
	test_one(Num::new(100, 0), Num::new(100, 0), Num::new(0, 0), None);
	test_one(Num::new(12345, 3), Num::new(12345, 0), Num::new(12332655, 0), None);
	test_one(Num::new(999888777666555, 0), Num::new(999888777666555, 0), Num::new(0, 0), None);
	test_one(Num::new(9998887776665558, 0), Num::new(9998887776665558, 0), Num::new(0, 0), None);
	test_one(Num::new(9998887776665558, 5), Num::new(9998887776665558, -3), Num::new(9998887676676680, 5), None); // Acutal: 999888767667668023334.442
	test_one(Num::new(-9998887776665558, 5), Num::new(-9998887776665558, -3), Num::new(-9998887676676680, 5), None); // Acutal: -999888767667668023334.442
}
#[test]
fn test_mul() {
	fn test_one(a: Num, b: Num, expected: Num) {
		let calc = a * b;
		println!("{} * {} = {} ({}) ? {}", a, b, calc, expected, calc == expected);
		assert_eq!(calc, expected);
	}
	
	println!(" == Testing Multiplication == ");
	test_one(Num::new(0, 0), Num::new(0, 0), Num::new(0, 0));
	test_one(Num::new(0, 0), Num::new(100, 0), Num::new(0, 0));
	test_one(Num::new(100, 0), Num::new(0, 0), Num::new(0, 0));
	test_one(Num::new(100, 0), Num::new(10, 0), Num::new(1000, 0));
	test_one(Num::new(10_000, 0), Num::new(10_000, 0), Num::new(100_000_000, 0));
	test_one(Num::new(123456789, 0), Num::new(123456789, 0), Num::new(15241578750190521, 0));
	test_one(Num::new(123456789, 0), Num::new(-123456789, 0), Num::new(-15241578750190521, 0));
	test_one(Num::new(-123456789, 0), Num::new(-123456789, 0), Num::new(15241578750190521, 0));
	test_one(Num::new(123456789123, 0), Num::new(123456789123, 0), Num::new(15241578774881880, 6)); // Actual answer is 15241578780560891, but due to rounding this becomes 15241578774881880000000
	test_one(Num::new(1255134, -5), Num::new(154, -2), Num::new(193290636, -7));
}
#[test]
fn test_div() {
	fn test_one(a: Num, b: Num, expected: Num) {
		let calc = a / b;
		//println!("{:?} / {:?} = {:?} ({:?}) ? {}", a, b, calc, expected, calc == expected);
		println!("{} / {} = {} ({}) ? {}", a, b, calc, expected, calc == expected);
		assert_eq!(calc, expected);
	}
	
	println!(" == Testing Division == ");
	test_one(Num::zero(), Num::new(1, 0), Num::zero());
	test_one(Num::new(1, 0), Num::new(2, 0), Num::new(5, -1));
	test_one(Num::new( 100, 0), Num::new( 200, 0), Num::new( 5, -1));
	test_one(Num::new(-100, 0), Num::new( 200, 0), Num::new(-5, -1));
	test_one(Num::new(-100, 0), Num::new(-200, 0), Num::new( 5, -1));
	test_one(Num::new( 100, 0), Num::new(-200, 0), Num::new(-5, -1));
	test_one(Num::new(1, 0), Num::new(2, 0), Num::new(5, -1));
	test_one(Num::new(9, 0), Num::new(3, 0), Num::new(3, 0));
	test_one(Num::new(100, 0), Num::new(2, 0), Num::new(50, 0));
	test_one(Num::new(1, 0), Num::new(3, 0), Num::new(3333333333333333, -16));
	test_one(Num::new(2, 0), Num::new(3, 0), Num::new(6666666666666667, -16));
	test_one(Num::new(3, 0), Num::new(3, 0), Num::new(1, 0));
	test_one(Num::new(4, 0), Num::new(3, 0), Num::new(1333333333333333, -15));
	test_one(Num::new(5, 0), Num::new(3, 0), Num::new(1666666666666667, -15));
	test_one(Num::new(6, 0), Num::new(3, 0), Num::new(2, 0));
	test_one(Num::new(12345, 0), Num::new(12345, -5), Num::new(1, 5));
	test_one(Num::new(682051, 0), Num::new(54, 0), Num::new(1263057407407407, -11));
	test_one(Num::new(942858765159, 0), Num::new(789145, 0), Num::new(1194785198105545, -9));
}

#[test]
fn test_floor() {
	fn test_one(num: Num, expected: Num) {
		let calc = num.floor();
		println!("{: >10}.floor() = {} ({}) ? {}", num, calc, expected, calc == expected);
		assert_eq!(calc, expected);
	}
	
	println!(" == Testing Flooring == ");
	test_one(Num::zero(), Num::zero());
	test_one(Num::new(123, -2), Num::new(1, 0));
	test_one(Num::new(156, -2), Num::new(1, 0));
	test_one(Num::new(199, -2), Num::new(1, 0));
	test_one(Num::new(-1, 0), Num::new(-1, 0));
	test_one(Num::new(-110, -2), Num::new(-1, 0));
	test_one(Num::new(-150, -2), Num::new(-1, 0));
	test_one(Num::new(-199, -2), Num::new(-1, 0));
}

#[test]
fn test_from_str() {
	fn test_one(input: &str, expected: Option<Num>) {
		let calc: Option<Num> = input.parse().ok();
		//println!("{} => {:?}", input, input.parse::<f64>());
		print!("{: >20} => ", input);
		if calc.is_some() {
			print!("Some({}) (", calc.unwrap());
		} else {
			print!("None (");
		}
		if expected.is_some() {
			print!("Some({})) ? ", expected.unwrap());
		} else {
			print!("None) ? ");
		}
		println!("{}", calc == expected);
		assert_eq!(calc, expected);
	}
	
	test_one("", None);
	test_one("1.23.4", None);
	test_one("1.56e51e41", None);
	test_one(".e", None);
	test_one("1.e", None);
	test_one("-1.e", None);
	test_one("1.e-", None);
	test_one("1e-1", Some(Num::new(1, -1)));
	test_one("1e15555555", None);
	test_one("-1e1", Some(Num::new(-1, 1)));
	test_one("0.0e0", Some(Num::zero()));
	test_one("1.23456", Some(Num::new(123456, -5)));
	test_one("123456", Some(Num::new(123456, 0)));
	test_one("1", Some(Num::new(1, 0)));
	test_one("-1", Some(Num::new(-1, 0)));
	test_one("1.", Some(Num::new(1, 0)));
	test_one("1e9", Some(Num::new(1, 9)));
	test_one("000000.5", Some(Num::new(5, -1)));
	test_one(".5000000", Some(Num::new(5, -1)));
	test_one(".0000000", Some(Num::zero()));
	test_one(".5", Some(Num::new(5, -1)));
	test_one(".5e4", Some(Num::new(5, 3)));
	test_one("99999999999999999999999", Some(Num::new(1, 23)));
	test_one("1.444444444444444444444", Some(Num::new(1444444444444444, -15)));
	test_one("1.555555555555555555555", Some(Num::new(1555555555555556, -15)));
}
