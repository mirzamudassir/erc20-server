use std::{error::Error, fmt};
use serde::{Deserialize, Serialize};
use sqlx::types::{Uuid};
use sqlx::FromRow;
use crate::{db};
use crate::ComnAddr;
use chrono::{DateTime, Utc};
use secp256k1::PublicKey;


#[derive(FromRow, Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct Addr {
	pub id: Uuid,
	pub name: Option<String>,
	pub comment: Option<String>,
	pub created: DateTime<Utc>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AddrRes {
	pub addr: ComnAddr,
	pub name: Option<String>,
	pub created: DateTime<Utc>,
}

impl From<Addr> for AddrRes {
  fn from(a: Addr) -> Self {
    Self {
      addr: ComnAddr::from_uuid(&a.id.to_string()).unwrap(),
      name: a.name,
      created: a.created
    }
  }
}

// fields are filters. 
#[derive(Serialize, Deserialize, Debug)]
pub struct AddrFilter {
	pub name: Option<String>,
	pub addr: Option<ComnAddr>,
	pub keys: Option<Vec<PublicKey>>,
	pub result: Option<Vec<Addr>>,
}

#[derive(Debug)]
pub enum AddrFilterErr {
	BadData,
	NotFound,
}

impl Error for AddrFilterErr {}

impl fmt::Display for AddrFilterErr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            AddrFilterErr::BadData => write!(f, "addr or name should be present."),
            AddrFilterErr::NotFound => write!(f, "no addr found"),
        }
    }
}

impl AddrFilter {
	pub async fn init(&mut self) -> Result<(), AddrFilterErr> {
		self.fetch().await?;
		Ok(())
	}

	pub fn to_res(&mut self) -> Vec<AddrRes> {
		let r: Vec<AddrRes> = self.result.clone().unwrap().into_iter().map(|r| AddrRes::from(r)).collect();
		r
	}

	async fn fetch(&mut self) -> Result<(), AddrFilterErr> {
		if self.keys == None {
			self.verify_addr().await?;
		} else {
			self.fetch_addrs_using_key().await?;
		}
		if self.result==None || self.result.as_ref().unwrap().len()==0 {
			self.result = None;
			return Err(AddrFilterErr::NotFound);
		}
		Ok(())
	}

	async fn verify_addr(&mut self) -> Result<(), AddrFilterErr>{
		let mut tx = db::db().await.begin().await.unwrap();
		let addr_uuid: String;
		if let Some(addr) = &self.addr {
			addr_uuid = addr.to_uuid();
		} else {
			if self.name == None {
				return Err(AddrFilterErr::BadData);
			}
			addr_uuid = "".to_string();
		}
		let data = sqlx::query_as::<_, Addr>(
			"
			SELECT a.id, a.created, a.name, a.comment
			FROM addr a 
			WHERE 
			CASE
				WHEN $1 != '' THEN a.id = $1::uuid
				WHEN $2 != '' THEN a.name = $2
			END
			",
		)
		.bind(addr_uuid)
		.bind(self.name.clone().unwrap_or("".to_string()))
		.fetch_all(&mut *tx)
		.await
		.unwrap();

		self.result = Some(data);
		Ok(())
	}

	async fn fetch_addrs_using_key(&mut self) -> Result<(), AddrFilterErr> {
		let mut tx = db::db().await.begin().await.unwrap();
		let addr_uuid: String;
		if let Some(addr) = &self.addr {
			addr_uuid = addr.to_uuid();
		} else {
			addr_uuid = "".to_string();
		}
		let keys = self.keys
			.as_ref()
			.unwrap()
			.into_iter()
			.map(|x| x.serialize())
			.collect::<Vec<[u8; 33]>>();
		let data = sqlx::query_as::<_, Addr>(
			"
			SELECT nt.id, nt.created, nt.name, nt.comment
			FROM (
				SELECT a.*, ARRAY_AGG(k.pub_key) as pub_keys
				FROM addr a
				JOIN addr_key ak ON ak.addr_id = a.id
				JOIN key k on k.id = ak.key_id
				WHERE
				CASE
					WHEN $1 != '' THEN a.id = $1::uuid
					WHEN $2 != '' THEN a.name = $2
					ELSE true
				END
				GROUP BY a.id
			) as nt
			WHERE nt.pub_keys @> $3
			",
		)
		.bind(addr_uuid)
		.bind(self.name.clone().unwrap_or("".to_string()))
		.bind(keys)
		.fetch_all(&mut *tx)
		.await
		.unwrap();

		self.result = Some(data);
		Ok(())
	}

	// pub fn get_key() {}
	// pub fn get_keys() {}
	// pub async fn verify_key() {}
	// pub fn fetch_key_using_addr() {}
}