use std::fmt;
use uuid::Uuid;

struct BaseConvert {
    x: u128,
    base: String,
}

impl BaseConvert {
    fn from_u128(x: u128, base: &str) -> Result<Self, &'static str> {
        Ok(Self { x, base: base.to_string()})
    }
    fn from_uuid(uuid_str: &str, base: &str) -> Result<Self, &'static str> {
    	let uuid = Uuid::parse_str(uuid_str).unwrap();
        Ok(Self { x: uuid.as_u128(), base: base.to_string()})
    }
    fn to_uuid(&self) -> String {
    	return Uuid::from_u128(self.x).to_string();
    }
    fn new(addr: &str, base: &str) -> Result<Self, &'static str> {
    	let mut x: u128 = 0;
    	let radix = base.len() as u128;
    	for c in addr.chars() {
    		let modulo = base.find(c).unwrap_or(0) as u128;
            x = x * radix + modulo;
    	}

        Ok(Self { x, base: base.to_string()})
    }
    fn to_u128(&self) -> u128 {
    	return self.x;
    }
}

impl fmt::Display for BaseConvert {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    	let radix = self.base.len() as u128;
        let mut x = self.x;
        // Good for binary formatting of `u128`s
        let mut result = ['\0'; 128];
        let mut used = 0;

        loop {
            let m = x % radix;
            x /= radix;

            result[used] = self.base.chars().nth(m as usize).unwrap();
            used += 1;

            if x == 0 {
                break;
            }
        }
        for c in result[..used].iter().rev() {
            write!(f, "{}", c)?;
        }
        Ok(())
    }
}

const COMN_DIGITS: &str = "0123456789ACDEFGHJKLMNPRSTUVWXYZ";

#[cfg(test)]
mod tests {
	use super::*;
    #[test]
    fn from_u128() {
        assert_eq!(BaseConvert::from_u128(1234, "0123456789").unwrap().to_string(), "1234");
        assert_eq!(BaseConvert::from_u128(1_000_000_000_000_000_000_000_000_000,
								    COMN_DIGITS).unwrap().to_string(), "TV5SY9ZM407LM00000");
        assert_eq!(BaseConvert::from_u128(100_000_000_000_000_000_000_000_000_000_000_000_000,
								    COMN_DIGITS).unwrap().to_string(), "2C7E6AHPM6SJX0L2J280000000");
        assert_eq!(BaseConvert::from_u128(u128::MAX, COMN_DIGITS).unwrap().to_string(), "7ZZZZZZZZZZZZZZZZZZZZZZZZZ");
    }
    #[test]
    fn to_uuid() {
        assert_eq!(BaseConvert::from_u128(1_000_000_000_000_000_000_000_000_000, COMN_DIGITS)
        	       .unwrap().to_uuid(), "00000000-033b-2e3c-9fd0-803ce8000000");
        assert_eq!(BaseConvert::from_u128(100_000_000_000_000_000_000_000_000_000_000_000_000, COMN_DIGITS)
        	       .unwrap().to_uuid(), "4b3b4ca8-5a86-c47a-098a-224000000000");
        assert_eq!(BaseConvert::from_u128(u128::MAX, COMN_DIGITS).unwrap()
        	       .to_uuid(), "ffffffff-ffff-ffff-ffff-ffffffffffff");
    }
    #[test]
    fn new() {
    	assert_eq!(BaseConvert::from_u128(1_000_000_000_000_000_000_000_000_000,
                   COMN_DIGITS).unwrap().to_string(), "TV5SY9ZM407LM00000");
    	assert_eq!(BaseConvert::new("7ZZZZZZZZZZZZZZZZZZZZZZZZZ", COMN_DIGITS).unwrap().to_u128(), u128::MAX);
    }
    #[test]
    fn from_uuid() {
        assert_eq!(BaseConvert::from_uuid("ffffffff-ffff-ffff-ffff-ffffffffffff", COMN_DIGITS).unwrap()
        	       .to_uuid(), "ffffffff-ffff-ffff-ffff-ffffffffffff");
        assert_eq!(BaseConvert::from_uuid("00000000-033b-2e3c-9fd0-803ce8000000", COMN_DIGITS).unwrap()
        	       .to_u128(), 1_000_000_000_000_000_000_000_000_000);

    }
}


// struct ComnAddr {
// 	addr: u128
// }

// impl ComnAddr {
// 	fn from_u128(value: u128) {
// 		let mut base = BaseConvert::new(value, COMN_DIGITS);

// 	}
// }

