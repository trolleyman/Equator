
use consts::*;
pub use decimal::d128;

/// Rounds `x` to `dp` decimal places
///
/// # Examples:
/// ```
/// # #[macro_use]
/// # extern crate decimal
/// # use decimal::d128;
/// # fn main() {
/// assert_eq!(round_dp(d128!(1.2345678), 2), d128!(1.23));
/// assert_eq!(round_dp(d128!(1.2345678), 4), d128!(1.2346));
/// assert_eq!(round_dp(d128!(1.2345678), 9), d128!(1.2345678));
/// assert_eq!(round_dp(d128!(1.5221), 0), d128!(2));
/// assert_eq!(round_dp(d128!(314.15928), -1), d128!(310));
/// assert_eq!(round_dp(d128!(351.15928), -2), d128!(400));
/// # }
/// ```
pub fn round_dp(x: d128, dp: i32) -> d128 {
	#[inline(always)]
	fn truncate(x: d128) -> d128 {
		let frac = x % D1;
		if frac >= DP5 {
			x - frac + D1
		} else {
			x - frac
		}
	}
	if x.is_infinite() || x.is_nan() {
		return x;
	}
	let ddp = d128::from(dp);
	
	let rounded = truncate(x.scaleb(ddp)).scaleb(-ddp);
	// Now remove trailing zeros (idk how the hell this works...)
	rounded / d128!(1.000000000000000000000000000000000)
}

/// Performs `v!`
pub fn factorial(x: d128) -> d128 {
	// Calculate the factorial using an approximation to the gamma function if `x` is not an integer, or is less than 0.
	if x.is_integer() && x >= D1 {
		let mut i = D2;
		let mut acc = D1;
		while i <= x && !acc.is_infinite() {
			acc = acc * i;
			i = i + D1;
		}
		acc
	} else {
		gamma(x + D1)
	}
}

pub fn gamma(mut x: d128) -> d128 {
	// Using the Lanczos approximation - https://en.wikipedia.org/wiki/Lanczos_approximation
	let p = &[  d128!(676.5203681218851),   d128!(-1259.1392167224028),  d128!(771.32342877765313),
			   d128!(-176.61502916214059),     d128!(12.507343278686905), d128!(-0.13857109526572012),
				  d128!(9.9843695780195716e-6), d128!(1.5056327351493116e-7)];
	
	if x < DP5 {
		DPI / (sin(DPI*x) * gamma(D1-x))
	} else {
		x = x - D1;
		let mut y = d128!(0.99999999999980993);
		
		for (i, pval) in p.iter().enumerate() {
			y = y + *pval / (x + d128::from(i as u32) + D1);
		}
		
		let t = x + d128::from(p.len() as u32) - DP5;
		(D2*DPI).pow(DP5) * t.pow(x+DP5) * DE.pow(-t) * y
	}
}

const NUM_ITER: u32 = 10;

pub fn sin(x: d128) -> d128 {
	sin_precision(x, NUM_ITER)
}
pub fn sin_precision(mut x: d128, iters: u32) -> d128 {
	// Calculate based on taylor series. https://en.wikipedia.org/wiki/Sine#Series_definition
	// Sum fromm 0 to ∞ of ((-1)^n / (2n + 1)!) * x^(2n + 1)
	
	x = x % DPI;   // equals x
	if x == DPI || x == -DPI {
		return D0;
	}
	let mut fact = D1; // equals (2n + 1)!
	let mut it = D1;   // equals (2n + 1)
	let mut neg = D1;  // equals (-1)^n
	let mut xpow = x;  // equals x^(2n + 1)
	let mut sum = D0;
	
	for _i in 0..(iters - 1) {
		// Sum
		sum = sum + ((neg / fact) * xpow);
		
		//println!("i: {:<1}, fact: {:<8}, it: {:<8}, neg: {:<8}, xpow: {:<8}, sum: {:<8}", _i, fact, it, neg, xpow, sum);
		
		// Update vars for next loop
		neg = -neg;
		it = it + D1;
		fact = fact * it;
		it = it + D1;
		fact = fact * it;
		xpow = xpow * x * x;
	}
	sum = sum + ((neg / fact) * xpow);
	
	sum
}

pub fn cos(x: d128) -> d128 {
	cos_precision(x, NUM_ITER)
}
pub fn cos_precision(x: d128, iters: u32) -> d128 {
	// cos(x) = sin(π/2 + x)
	sin_precision(x + DPI2, iters)
}

pub fn tan(x: d128) -> d128 {
	tan_precision(x, NUM_ITER)
}
pub fn tan_precision(x: d128, iters: u32) -> d128 {
	// tan(x) = sin(x) / cos(x)
	sin_precision(x, iters) / cos_precision(x, iters)
}
