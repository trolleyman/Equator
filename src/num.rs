use std::ops::{Add, Sub, Mul, Div, Neg, Rem};
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

const POW_TABLE: [u64; MAX_PRECISION as usize] = [
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
		if exp >= MAX_PRECISION as i32 {
			panic!("exp cannot be higher than MAX_PRECISION");
		}
		num * POW_TABLE[exp as usize] as i64
	} else {
		if -exp >= MAX_PRECISION as i32 {
			return 0;
		}
		num / POW_TABLE[(-exp) as usize] as i64
	}
}
#[inline(always)]
fn pow_10(exp: u32) -> u64 {
	debug_assert!(POW_TABLE.len() == MAX_PRECISION as usize);
	if exp > MAX_PRECISION as u32 {
		panic!("expcannot be higher than MAX_PRECISION");
	}
	unsafe { *POW_TABLE.get_unchecked(exp as usize) }
}
#[inline]
fn digits_len(num: i64) -> u32 {
	for i in (0u32..MAX_PRECISION).rev() {
		if num / pow_10(i) as i64 != 0 {
			return i + 1u32;
		}
	}
	0
}

#[allow(dead_code)]
#[inline]
fn get_digit(num: u64, pos: u32) -> u32 {
	(num / pow_10(pos)) as u32 % 10
}

/// Performs the newton-raphson method with initial guess `init` and functions `f` and `f_dash`
/// Formula: x1 = x0 - (f(x0) / f_dash(x0))
/// Keeps on looping until abs(x0 - x1) == 0 or when 100 iterations have completed.
#[inline(always)]
fn newton_raphson<F, G>(init: Num, f: F, f_dash: G) -> Num where F: Fn(Num) -> Num, G: Fn(Num) -> Num {
	let mut x1 = init;
	let mut x0 = init;
	for _ in 0..100 {
		x1 = x0 - f(x0) / f_dash(x0);
		if (x1 - x0).is_zero() { // No change, close enough
			break;
		}
		x0 = x1;
	}
	x1
}

/// Base-10 number (Scientific notation)
/// The aim of this structure is not to be as fast as possible, but as precise as possible.
/// `1.2345` is represented with significand = 1,234,500,000,000,000 exponent = -15
#[derive(Copy, Clone)]
pub struct Num {
	// Significand
	sig: i64,
	// Exponent
	exp: i16,
}
impl Num {
	pub const E: Num = Num{sig:2718281828459045, exp:-15};//Num::new(27182818284590452, -16);
	pub const PI: Num = Num{sig:3141592653589793, exp:-15};//Num::new(31415926535897932, -16);
	pub const GOLDEN_RATIO: Num = Num{sig:1618033988749895, exp:-15};//Num::new(16180339887498948, -16);
	pub const LN_10: Num = Num{sig: 2302585092994046, exp:-15};//Num::new(3025850929940456, -16);
	
	#[inline(always)]
	pub fn new(sig: i64, exp: i16) -> Num {
		Num {sig:sig, exp:exp}.normalise()
	}
	
	#[inline(always)]
	pub fn sig(sig: i64) -> Num {
		Num {sig:sig, exp:0}.normalise()
	}
	
	#[inline(always)]
	pub fn zero() -> Num {
		Num{ sig:0, exp:0 }
	}
	
	pub fn normalise(mut self) -> Num {
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
				self = self.normalise();
			}
		}
		self
	}
	
	#[inline]
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
	
	#[inline]
	pub fn is_integer(self) -> bool {
		if self.exp >= 0 || self.is_zero() {
			true
		} else if self.exp < -(PRECISION as i16 - 1) {
			false
		} else {
			self - self.floor() == Num::zero()
		}
	}
	
	#[inline(always)]
	pub fn floor(self) -> Num {
		if self.exp >= 0 {
			self
		} else {
			Num::new(times_pow_10(times_pow_10(self.sig, self.exp as i32), -self.exp as i32), self.exp)
		}
	}
	
	// Functions
	// Trig functions all use radians.
	#[inline(always)]
	pub fn abs(self) -> Num {
		if self.is_negative() {
			-self
		} else {
			 self
		}
	}
	#[inline(always)]
	pub fn sqrt(self) -> Option<Num> {
		if self.is_negative() {
			None
		} else if self.is_zero() {
			Some(Num::zero())
		} else {
			// x = root(S), x^2 = S, x^2 - S = 0 ∴ f(x) = x^2 - S ∴ f'(x) = 2x
			let est = newton_raphson(self, move |x| { x * x - self }, move |x| { x * 2 });
			Some(est)
		}
	}
	
	#[inline(always)]
	pub fn sin(self) -> Option<Num> {
		self.as_float().sin().to_num_opt()
	}
	#[inline(always)]
	pub fn cos(self) -> Option<Num> {
		self.as_float().cos().to_num_opt()
	}
	#[inline(always)]
	pub fn tan(self) -> Option<Num> {
		self.as_float().tan().to_num_opt()
	}
	#[inline(always)]
	pub fn asin(self) -> Option<Num> {
		self.as_float().asin().to_num_opt()
	}
	#[inline(always)]
	pub fn acos(self) -> Option<Num> {
		self.as_float().acos().to_num_opt()
	}
	#[inline(always)]
	pub fn atan(self) -> Option<Num> {
		self.as_float().atan().to_num_opt()
	}
	
	#[inline(always)]
	pub fn sinh(self) -> Option<Num> {
		self.as_float().sinh().to_num_opt()
	}
	#[inline(always)]
	pub fn cosh(self) -> Option<Num> {
		self.as_float().cosh().to_num_opt()
	}
	#[inline(always)]
	pub fn tanh(self) -> Option<Num> {
		self.as_float().tanh().to_num_opt()
	}
	#[inline(always)]
	pub fn asinh(self) -> Option<Num> {
		self.as_float().asinh().to_num_opt()
	}
	#[inline(always)]
	pub fn acosh(self) -> Option<Num> {
		self.as_float().acosh().to_num_opt()
	}
	#[inline(always)]
	pub fn atanh(self) -> Option<Num> {
		self.as_float().atanh().to_num_opt()
	}
	
	pub fn ln(mut self) -> Option<Num> {
		if self <= Num::zero() {
			None
		} else if self == Num::new(1, 0) {
			Some(Num::zero())
		} else if self == Num::E {
			Some(Num::sig(1))
		} else {
			// https://en.wikipedia.org/wiki/Natural_logarithm#Numerical_value
			// ln(x) = ln((1+y)/(1-y)) = 2y( 1/1 + 1/3 * y^2 + 1/5 * y^4 + 1/7 * y^6) )
			
			// ln(a * 10^b) = ln(a) + b * ln(10)
			let exp = self.exp + PRECISION as i16 - 1;
			//print!("{}", self);
			self.exp = - (PRECISION as i16 - 1);
			//println!(" = {} * 10^{}", self, exp);
			
			let y = (self - 1) / (self + 1);
			
			let mut res = 1.to_num();
			let mut prev = res;
			let mut y_cache = y * y;
			//println!("y: {}", y);
			//println!("y_cache: {}", y_cache);
			//println!("res: {}", res);
			// Starting after 1/1
			for i in 0..100 {
				res = res + ((Num::sig(1)/(i.to_num() * Num::sig(2) + Num::sig(3))) * y_cache);
				//println!("res: {}", res);
				y_cache = y_cache * y * y;
				
				// No difference in result this iteration, so break.
				if (res - prev).abs() == Num::zero() {
					//println!("break;");
					break;
				}
				prev = res;
			}
			res = res * (Num::sig(2) * y);
			
			res = res + (exp.to_num() * Num::LN_10);
			
			Some(res)
		}
	}
	
	#[inline(always)]
	pub fn recip(self) -> Option<Num> {
		if self.is_zero() {
			None
		} else {
			Some(Num::new(1, 0) / self)
		}
	}
	
	#[inline(always)]
	pub fn factorial(mut self) -> Option<Num> {
		if !self.is_integer() || self.is_negative() {
			return None;
		}
		let mut res = self;
		if self.is_zero() {
			return Some(Num::new(1, 0));
		}
		self = self - 1;
		loop {
			if self.is_zero() {
				break;
			}
			res = res * self;
			self = self - 1;
		}
		Some(res)
	}
	
	pub fn pow(self, mut n: Num) -> Num {
		// https://en.wikipedia.org/wiki/Exponentiation_by_squaring#Basic_method
		// The iterative method
		let mut x = self;
		
		if n < Num::zero() {
			x = Num::sig(1) / x;
			n = -n;
		}
		if n.is_zero() {
			return Num::sig(1);
		}
		let mut y = Num::sig(1);
		while n > Num::sig(1) {
			if (n % Num::sig(2)).is_zero() {
				// Even
				x = x * x;
				n = n / Num::sig(2);
			} else {
				// Odd
				y = x * y;
				x = x * x;
				n = (n-Num::sig(1)) / Num::sig(2);
			}
		}
		x * y
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
			//println!("sig: {}, self.exp: {}, before: {}", sig, self.exp, before_dp);
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

impl Num {
	fn add_precise(mut self, mut rhs: Num) -> Num {
		if self.is_zero() { // Avoids dropping precision when one of the numbers is zero.
			return rhs;
		} else if rhs.is_zero() {
			return self;
		}
		if self.exp > rhs.exp {
			self.sig *= 10; // Increase precision by 1
			self.exp -= 1;
			self.sig += times_pow_10(rhs.sig, -(self.exp as i32 - rhs.exp as i32));
			self
		} else {
			rhs.sig *= 10;
			rhs.exp -= 1;
			rhs.sig += times_pow_10(self.sig, -(rhs.exp as i32 - self.exp as i32));
			rhs
		}
	}
}
impl Add<Num> for Num {
	type Output = Num;
	#[inline(always)]
	fn add(self, rhs: Num) -> Num {
		Num::add_precise(self, rhs).normalise()
	}
}
impl Sub<Num> for Num {
	type Output = Num;
	#[inline(always)]
	fn sub(self, rhs: Num) -> Num {
		self + (-rhs)
	}
}

impl Mul<Num> for Num {
	type Output = Num;
	fn mul(mut self, mut rhs: Num) -> Num {
		if self.is_zero() || rhs.is_zero() {
			return Num::zero();
		}
		
		self = self.normalise();
		rhs  = rhs .normalise();
		
		//println!("  {:e}", self);
		//println!("* {:e}", rhs);
		//println!("---------------------");
		
		let mut res = Num::zero();
		let base_exp = self.exp + rhs.exp;
		//println!("base: {}", base_exp);
		for i in 0..PRECISION {
			// Get digit then shift self.sig to right. This way we get each digit in turn.
			let digit = self.sig % 10;
			self.sig /= 10;
			let new = Num::new(digit * rhs.sig, base_exp + (i as i16));
			//println!("{} * {} ({: >17})e{: <3} -> {: >20} {:?}, res: {}", digit, rhs.sig, digit * rhs.sig, base_exp + (i as i16), new, new, res);
			res = res + new;
		}
		res
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
			result = result + mul * pow_10(i) as i64;
			a -= b * mul;
			b /= 10;
			i -= 1;
			//println!("mul: {: >20}, a: {: >20}, b: {: >20}, result: {:?}", mul, a, b, result);
		}
		
		Num::new(result, ret_exp)
		
		/* // Non-long division - low precision
		let ret_exp = self.exp - rhs.exp - PRECISION as i16 - 1;
		//let rem = (self.sig % rhs.sig);
		let ret_sig = ((self.sig * 1000) / (rhs.sig / 100000000000000));// + rem;
		
		println!("ret: {:?}", Num{sig:ret_sig, exp:ret_exp});
		Num::new(ret_sig, ret_exp)*/
	}
}
impl Rem<Num> for Num {
	type Output = Self;
	fn rem(self, rhs: Num) -> Num {
		let a = (self / rhs).floor();
		//println!("self {} % rhs {} --> {}", self, rhs, a);
		self - (a * rhs)
	}
}

impl<T> Add<T> for Num where T: ToNum {
	type Output = Num;
	fn add(self, rhs: T) -> Num {
		self + rhs.to_num()
	}
}
impl<T> Sub<T> for Num where T: ToNum {
	type Output = Num;
	fn sub(self, rhs: T) -> Num {
		self - rhs.to_num()
	}
}

impl<T> Mul<T> for Num where T: ToNum {
	type Output = Num;
	fn mul(self, rhs: T) -> Num {
		self * rhs.to_num()
	}
}
impl<T> Div<T> for Num where T: ToNum {
	type Output = Num;
	fn div(self, rhs: T) -> Num {
		self / rhs.to_num()
	}
}

pub trait ToNumOpt {
	fn to_num_opt(self) -> Option<Num>;
}
pub trait ToNum {
	fn to_num(self) -> Num;
}
impl<T> ToNumOpt for T where T: ToNum {
	fn to_num_opt(self) -> Option<Num> {
		Some(self.to_num())
	}
}

impl ToNumOpt for f32 {
	fn to_num_opt(self) -> Option<Num> {
		if self.is_nan() || self.is_infinite() {
			return None;
		}
		Num::from_str(&format!("{}", self)).ok()
	}
}
impl ToNumOpt for f64 {
	fn to_num_opt(self) -> Option<Num> {
		if self.is_nan() || self.is_infinite() {
			return None;
		}
		Num::from_str(&format!("{}", self)).ok()
	}
}

impl ToNum for i64 {
	#[inline(always)]
	fn to_num(self) -> Num {
		Num::new(self, 0)
	}
}
impl ToNum for u64 {
	#[inline(always)]
	fn to_num(self) -> Num {
		Num::new(self as i64, 0)
	}
}
impl ToNum for i32 {
	#[inline(always)]
	fn to_num(self) -> Num {
		Num::new(self as i64, 0)
	}
}
impl ToNum for u32 {
	#[inline(always)]
	fn to_num(self) -> Num {
		Num::new(self as i64, 0)
	}
}
impl ToNum for i16 {
	#[inline(always)]
	fn to_num(self) -> Num {
		Num::new(self as i64, 0)
	}
}
impl ToNum for u16 {
	#[inline(always)]
	fn to_num(self) -> Num {
		Num::new(self as i64, 0)
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
			ret_sig += match c.to_digit(10) { Some(v) => v as i64, None => return Err(NumParseError) } * pow_10(cur_pow) as i64;
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
		// 1.4200	 1420000000000000 -15
		// 000.0024 2400000000000000 -18
		
		let num = Num::new(ret_sig, ret_exp + extra_exp - PRECISION as i16 - 2);
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
fn test_to_num() {
	fn test_one<T>(f: T, n: Option<Num>) where T: ToNumOpt + Display + Clone {
		let calc = f.clone().to_num_opt();
		
		println!("{} = {:?} ({:?}) ? {}", f, calc, n, calc == n);
		assert_eq!(calc, n);
	}
	
	println!(" == Testing ToNum == ");
	test_one(1.2, Some(Num::new(12, -1)));
	test_one(::std::f64::INFINITY, None);
	test_one(1245506125, Some(Num::new(1245506125, 0)));
}

#[test]
fn test_is_integer() {
	fn test_one(n: Num, expected: bool) {
		let calc = n.is_integer();
		println!("{}.is_integer() = {} ({}) ? {}", n, calc, expected, calc == expected);
		assert_eq!(calc, expected);
	}
	
	test_one(Num::new(12, -1), false);
	test_one(Num::new(12, 0), true);
	test_one(Num::new(12, 100), true);
	test_one(Num::new(-12, 0), true);
	test_one(Num::new(1200000, -5), true);
	test_one(Num::new(12, -3), false);
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
fn test_get_digit() {
	fn test_one(num: u64, pos: u32, expected: u32) {
		let calc = get_digit(num, pos);
		
		println!("{} @ {} = {} ({}) ? {}", num, pos, calc, expected, calc == expected);
		assert_eq!(expected, calc);
	}
	
	println!(" == Testing get_digit == ");
	test_one(1, 0, 1);
	test_one(0, 0, 0);
	test_one(0, 10, 0);
	test_one(10, 1, 1);
	test_one(10, 0, 0);
	test_one(12345, 0, 5);
	test_one(12345, 1, 4);
	test_one(12345, 2, 3);
	test_one(12345, 3, 2);
	test_one(12345, 4, 1);
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
	test_one(Num::new(462, -6), "4.62e-4", "0.000462");
	test_one(Num::new(-100, 0), "-1e2", "-100");
	test_one(Num::new(9999999999999999	, 0), "9.999999999999999e15", "9999999999999999"); // 16 9s
	test_one(Num::new(99999999999999989 , 0), "9.999999999999999e16", "99999999999999990"); // 16 9s, 1 8
	test_one(Num::new(99999999999999999 , 0), "1e17", "100000000000000000"); // 17 9s
	test_one(Num::new(-9999999999999999 , 0), "-9.999999999999999e15", "-9999999999999999"); // 16 9s
	test_one(Num::new(-99999999999999989, 0), "-9.999999999999999e16", "-99999999999999990"); // 16 9s, 1 8
	test_one(Num::new(-99999999999999999, 0), "-1e17", "-100000000000000000"); // 17 9s
	test_one(Num::new(922337203685477580 , 0), "9.223372036854776e17", "922337203685477600"); // Demonstration that extra precision is discarded.
	test_one(Num::new(9223372036854775806, 0), "9.223372036854776e18", "9223372036854776000");
	test_one(Num::new(123456789, 45), "1.23456789e53", "123456789000000000000000000000000000000000000000000000");
	test_one(Num::new(987654456789, -6), "9.87654456789e5", "987654.456789");
	
	test_one(Num::E           , "2.718281828459045e0", "2.718281828459045");
	test_one(Num::PI          , "3.141592653589793e0", "3.141592653589793");
	test_one(Num::GOLDEN_RATIO, "1.618033988749895e0", "1.618033988749895");
}
#[test]
fn test_cmp() {
	fn test_one(a: Num, b: Num, expected: Ordering) {
		let cmp_s = match expected {
			Ordering::Less		 => "<",
			Ordering::Equal		=> "=",
			Ordering::Greater	=> ">",
		};
		let calc = a.cmp(&b);
		println!("{:?} {} {:?} <=> {: >20} {} {: <20} ? {}", a, cmp_s, b, a, cmp_s, b, calc == expected);
		assert_eq!(expected, calc);
	}
	
	println!(" == Testing Comparisons == ");
	test_one(Num::new(0, 0), Num::new(0,	10), Ordering::Equal);
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
	test_one(Num::new(999888777666555, 0)	, Num::new(999888777666555, 0)	 , Num::new(0, 0), None);
	test_one(Num::new(9998887776665558, 0) , Num::new(9998887776665558, 0)	, Num::new(0, 0), None);
	test_one(Num::new(9998887776665558, 5) , Num::new(9998887776665558, -3) , Num::new(9998887676676680, 5), None); // Acutal: 999888767667668023334.442
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
	test_one(Num::new(123456789123, 0), Num::new(123456789123, 0), Num::new(1524157878056089, 7));
	// ^ Actual answer is 15241578780560891109129, but due to rounding this becomes 15241578780560890000000
	test_one(Num::new(1255134, -5), Num::new(154, -2), Num::new(193290636, -7));
	test_one(Num::new(9999999999999999, 0), Num::new(9999999999999999, 0), Num::new(9999999999999998, 16));
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
fn test_rem() {
	fn test_one(a: Num, b: Num, expected: Num) {
		let calc = a % b;
		//println!("{:?} % {:?} = {:?} ({:?}) ? {}", a, b, calc, expected, calc == expected);
		println!("{} % {} = {} ({}) ? {}", a, b, calc, expected, calc == expected);
		assert_eq!(calc, expected);
	}
	
	println!(" == Testing Modulo (Remainder) == ");
	test_one(Num::zero(), Num::new(1, 0), Num::zero());
	test_one(Num::new(1, 0), Num::new(2, 0), Num::new(1, 0));
	test_one(Num::new( 100, 0), Num::new( 2, 0), Num::zero());
	test_one(Num::new(-100, 0), Num::new( 2, 0), Num::zero());
	test_one(Num::new(-100, 0), Num::new(-2, 0), Num::zero());
	test_one(Num::new( 100, 0), Num::new(-2, 0), Num::zero());
	test_one(Num::new( 101, 0), Num::new( 2, 0), Num::sig( 1));
	test_one(Num::new(-101, 0), Num::new( 2, 0), Num::sig(-1));
	test_one(Num::new(-101, 0), Num::new(-2, 0), Num::sig(-1));
	test_one(Num::new( 101, 0), Num::new(-2, 0), Num::sig( 1));
}

#[test]
fn test_ln() {
	fn test_one(num: Num, expected: Option<Num>) {
		let calc = num.ln();
		
		print!("ln({}) = ", num);
		match calc {
			Some(n) => print!("Some({}) ", n),
			None    => print!("None ")
		}
		
		match expected {
			Some(n) => println!("(Some({})) ? {}", n, calc == expected),
			None    => println!("(None) ? {}", calc == expected)
		}
		assert_eq!(calc, expected);
	}
	
	println!(" == Testing Ln == ");
	test_one(Num::zero(), None);
	test_one(Num::new(-1, 0), None);
	test_one(Num::new(-40, -4), None);
	test_one(Num::new(1, 0), Some(Num::zero()));
	test_one(Num::E, Some(Num::new(1, 0)));
	test_one(Num::new(5124151, 0), Some(Num::new(15449475410729269, -15))); // ln(5124151) == 15.449475410729269678456407320185863564245570787943220
	test_one(Num::new(5, -1), Some(Num::new(-693147180559944, -15))); // actually -0.6931471805599453, but inaccurate so calculates -0.693147180559944
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
