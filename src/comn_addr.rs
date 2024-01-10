use once_cell::sync::Lazy;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::collections::HashMap;
use std::fmt;
use uuid::Uuid;

static _COMN_DIGITS: Lazy<HashMap<char, char>> = Lazy::new(|| {
	HashMap::from([
		('0', '0'),
		('1', '1'),
		('2', '2'),
		('3', '3'),
		('4', '4'),
		('5', '5'),
		('6', '6'),
		('7', '7'),
		('8', '8'),
		('9', '9'),
		('A', 'A'),
		('B', '8'),
		('C', 'C'),
		('D', 'D'),
		('E', 'E'),
		('F', 'F'),
		('G', 'G'),
		('H', 'H'),
		('I', '1'),
		('J', 'J'),
		('K', 'K'),
		('L', 'L'),
		('M', 'M'),
		('N', 'N'),
		('O', '0'),
		('P', 'P'),
		('Q', '0'),
		('R', 'R'),
		('S', 'S'),
		('T', 'T'),
		('U', 'U'),
		('V', 'V'),
		('W', 'W'),
		('X', 'X'),
		('Y', 'Y'),
		('Z', 'Z'),
		('a', 'A'),
		('b', '8'),
		('c', 'C'),
		('d', 'D'),
		('e', 'E'),
		('f', 'F'),
		('g', 'G'),
		('h', 'H'),
		('i', '1'),
		('j', 'J'),
		('k', 'K'),
		('l', 'L'),
		('m', 'M'),
		('n', 'N'),
		('o', '0'),
		('p', 'P'),
		('q', '0'),
		('r', 'R'),
		('s', 'S'),
		('t', 'T'),
		('u', 'U'),
		('v', 'V'),
		('w', 'W'),
		('x', 'X'),
		('y', 'Y'),
		('z', 'Z'),
	])
});

const COMN_DIGITS: &str = "0123456789ACDEFGHJKLMNPRSTUVWXYZ";

#[derive(Debug, Clone, PartialEq)]
pub struct ComnAddr {
	addr: String,
}

impl ComnAddr {
	pub fn from_u128(x: u128) -> Result<Self, &'static str> {
		Ok(Self { addr: u128_to_addr(x) })
	}
	pub fn from_uuid(uuid_str: &str) -> Result<Self, &'static str> {
		let uuid = Uuid::parse_str(uuid_str).unwrap();
		let addr = u128_to_addr(uuid.as_u128());
		Ok(Self { addr })
	}
	pub fn to_uuid(&self) -> String {
		return Uuid::from_u128(self.to_u128()).to_string();
	}
	pub fn new(addr: &str) -> Result<Self, &'static str> {
		let mut chars = addr.chars();
		if Some('≈') != chars.next() {
			panic!("Not a comn address={}", addr);
		}

		Ok(Self { addr: chars.collect::<String>() })
	}

	pub fn to_u128(&self) -> u128 {
		let mut x: u128 = 0;
		let radix = COMN_DIGITS.len() as u128;
		for c in self.addr.chars() {
			if let Some(_c) = _COMN_DIGITS.get(&c) {
				if let Some(modulo) = COMN_DIGITS.find(*_c) {
					x = x * radix + modulo as u128;
				} else {
					panic!("Not a valid char={}", c);
				}
			} else {
				panic!("Not a valid char={}", c);
			}
		}
		x
	}
}

fn u128_to_addr(x: u128) -> String {
	// Good for binary formatting of `u128`s
	let mut _x = x;
	let radix = COMN_DIGITS.len() as u128;
	let mut result = ['\0'; 26];
	let mut used = 0;
	loop {
		let m = _x % radix;
		_x /= radix;
		result[used] = COMN_DIGITS.chars().nth(m as usize).unwrap();
		used += 1;
		if _x == 0 {
			break;
		}
	}
	let out: String = result.to_vec().into_iter().rev().collect();
	out.trim_start_matches('\0').to_string()
}

impl fmt::Display for ComnAddr {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
			write!(f, "≈{}", self.addr)?;
		Ok(())
	}
}

impl Serialize for ComnAddr {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
    	// println!("{}", self.to_string());
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for ComnAddr {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let n = String::deserialize(deserializer)?;

        ComnAddr::new(&n).map_err(serde::de::Error::custom)
    }
}

impl Into<Uuid> for ComnAddr {
	fn into(self) -> Uuid {
		Uuid::from_u128(self.to_u128())
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	#[test]
	fn from_u128() {
		assert_eq!(
			ComnAddr::from_u128(1_000_000_000_000_000_000_000_000_000)
				.unwrap()
				.to_string(),
			"≈TV5SY9ZM407LM00000"
		);
		assert_eq!(
			ComnAddr::from_u128(100_000_000_000_000_000_000_000_000_000_000_000_000)
				.unwrap()
				.to_string(),
			"≈2C7E6AHPM6SJX0L2J280000000"
		);
		assert_eq!(
			ComnAddr::from_u128(u128::MAX).unwrap().to_string(),
			"≈7ZZZZZZZZZZZZZZZZZZZZZZZZZ"
		);
	}
	#[test]
	fn to_uuid() {
		assert_eq!(
			ComnAddr::new("≈1").unwrap().to_uuid(),
			"00000000-0000-0000-0000-000000000001"
		);
		assert_eq!(
			ComnAddr::from_u128(1_000_000_000_000_000_000_000_000_000)
				.unwrap()
				.to_uuid(),
			"00000000-033b-2e3c-9fd0-803ce8000000"
		);
		assert_eq!(
			ComnAddr::from_u128(100_000_000_000_000_000_000_000_000_000_000_000_000)
				.unwrap()
				.to_uuid(),
			"4b3b4ca8-5a86-c47a-098a-224000000000"
		);
		assert_eq!(
			ComnAddr::from_u128(u128::MAX).unwrap().to_uuid(),
			"ffffffff-ffff-ffff-ffff-ffffffffffff"
		);
	}
	#[test]
	fn new() {
		assert_eq!(
			ComnAddr::from_u128(1_000_000_000_000_000_000_000_000_000)
				.unwrap()
				.to_string(),
			"≈TV5SY9ZM407LM00000"
		);
		assert_eq!(
			ComnAddr::new("≈7ZZZZZZZZZZZZZZZZZZZZZZZZZ")
				.unwrap()
				.to_u128(),
			u128::MAX
		);
	}
	#[test]
	fn from_uuid() {
		assert_eq!(
			ComnAddr::from_uuid("ffffffff-ffff-ffff-ffff-ffffffffffff")
				.unwrap()
				.to_uuid(),
			"ffffffff-ffff-ffff-ffff-ffffffffffff"
		);
		assert_eq!(
			ComnAddr::from_uuid("00000000-033b-2e3c-9fd0-803ce8000000")
				.unwrap()
				.to_u128(),
			1_000_000_000_000_000_000_000_000_000
		);
	}
}
