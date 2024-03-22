use core::panic;

use regex::Regex;

pub fn into_milliseconds(duration: &str) -> u128 {
	let re = Regex::new(r"(\d+)(ms|[smhd])").unwrap();
	let Some(duration_vec) = re.captures(duration) else {
		panic!("Unable to parse window size:{}", duration);
	};

	let value = duration_vec[1].parse().unwrap();
	let unit = &duration_vec[2];

	match unit {
		"ms" => value,
		"s" => value * 1000,
		"m" => value * 1000 * 60,
		"h" => value * 1000 * 60 * 60,
		"d" => value * 1000 * 60 * 60 * 24,
		_ => panic!("Unable to parse window size:{}", duration),
	}
}

// #[cfg(test)]
// mod tests {
// 	use super::into_milliseconds;

// 	#[test]
// 	fn test_days_into_milliseconds() {
// 		assert_eq!(86400000, into_milliseconds("1d"))
// 	}

// 	#[test]
// 	fn test_hours_into_milliseconds() {
// 		assert_eq!(7200000, into_milliseconds("2h"))
// 	}

// 	#[test]
// 	fn test_minutes_into_milliseconds() {
// 		assert_eq!(300000, into_milliseconds("5m"))
// 	}

// 	#[test]
// 	fn test_seconds_into_milliseconds() {
// 		assert_eq!(10000, into_milliseconds("10s"))
// 	}

// 	#[test]
// 	fn test_milliseconds() {
// 		assert_eq!(2000000, into_milliseconds("2000000ms"))
// 	}
// }
