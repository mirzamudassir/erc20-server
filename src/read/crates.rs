use std::{error::Error, fmt};
use serde::{Deserialize, Serialize};
use sqlx::types::Uuid;
use secp256k1::PublicKey;
use crate::{
	db, AccessType, Crate,
	ComnAddr
};

#[derive(Debug)]
pub enum CrateFilterErr {
	BadData,
	NotFound,
	Multiple,
}

impl Error for CrateFilterErr {}

impl fmt::Display for CrateFilterErr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            CrateFilterErr::BadData => write!(f, "addr or name missing"),
            CrateFilterErr::NotFound => write!(f, "no crate found"),
            CrateFilterErr::Multiple => write!(f, "multiple create exists"),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CrateOwnerFilter {
	pub name: Option<String>,
	pub addr: Option<String>,
	pub crate_ids: Option<Vec<Uuid>>,
}


impl CrateOwnerFilter {
	pub async fn get_ids(&mut self) -> Result<Vec<Uuid>, CrateFilterErr> {
		if self.crate_ids == None {
			self.fetch().await?;
		}
		Ok(self.crate_ids.clone().unwrap())
	}
	pub async fn get_id(&mut self) -> Result<Uuid, CrateFilterErr> {
		if self.crate_ids == None {
			self.fetch().await?;
		}
		if self.crate_ids.clone().unwrap().len()>1 {
			return Err(CrateFilterErr::Multiple);
		}
		Ok(self.crate_ids.clone().unwrap()[0])
	}
	async fn fetch(&mut self) -> Result<(), CrateFilterErr> {
		let mut tx = db::db().await.begin().await.unwrap();
		if let Some(addr) = &self.addr {
			if let Ok(crate_ids) = sqlx::query_scalar::<_, Uuid>(
				"
				SELECT c.id
				FROM crate c
				JOIN crate_access ca ON ca.crate_id = c.id
				JOIN addr a ON ca.addr_id = a.id
				WHERE a.id = $1::uuid AND
				CASE
					WHEN $2 != '' THEN c.name = $2 ELSE true
				END
				"
			)
			.bind(ComnAddr::new(& addr).unwrap().to_uuid())
			.bind(self.name.clone().unwrap_or("".to_string()))
			.fetch_all(&mut *tx)
			.await {
				if crate_ids.len()==0 {
					return Err(CrateFilterErr::NotFound);
				}
				self.crate_ids = Some(crate_ids);
			} else {
				return Err(CrateFilterErr::NotFound);
			}
		} else {
			return Err(CrateFilterErr::BadData);
		}
		Ok(())
	}
}



#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct CrateFilter {
	pub name: Option<String>,
	pub addr: Option<Vec<ComnAddr>>,
	pub pub_key: Option<PublicKey>,
	pub crate_id: Option<String>,
	pub access_type: Vec<AccessType>,
	// pub per_page: usize,
	// pub page_no: usize,
	pub result: Option<Vec<Crate>>
}


impl CrateFilter {
	pub async fn init(&mut self) -> Result<(), CrateFilterErr> {
		self.fetch().await?;
		Ok(())
	}
	pub fn get_crate(&mut self) -> Result<Vec<Crate>, CrateFilterErr> {
		if self.result.clone().unwrap().len()>1 {
			return Err(CrateFilterErr::Multiple);
		}
		Ok(self.result.clone().unwrap())
	}
	async fn fetch(&mut self) -> Result<(), CrateFilterErr> {
		if self.pub_key == None {
			self.fetch_using_addr().await?;
		} else {
			self.fetch_using_key().await?;
		}
		Ok(())
	}
	async fn fetch_using_key(&mut self) -> Result<(), CrateFilterErr> {
		let mut tx = db::db().await.begin().await.unwrap();
		let addrs = self.addr.clone().unwrap_or(Vec::new()).into_iter()
			.map(|x| x.to_uuid()).collect::<Vec<String>>();
		if let Ok(crates) = sqlx::query_as::<_, Crate>(
			"
			SELECT DISTINCT ON (c.id) c.*
			FROM crate c
			JOIN crate_access ca ON ca.crate_id = c.id
			JOIN addr a ON ca.addr_id = a.id
			LEFT JOIN addr_key ak ON ak.addr_id = a.id
			LEFT JOIN key k on k.id = ak.key_id
			WHERE (k.pub_key = $1 OR a.id = ANY($2::uuid[]) )
			AND ca.type = ANY($3)
			AND 
			CASE
				WHEN $4 != '' THEN c.id = $4::uuid
				WHEN $5 != '' THEN c.name = $5
				ELSE true
			END
			"
		)
		.bind(self.pub_key.unwrap().serialize())
		.bind(addrs)
		.bind(self.access_type.clone())
		.bind(self.crate_id.clone().unwrap_or("".to_string()))
		.bind(self.name.clone().unwrap_or("".to_string()))
		.fetch_all(&mut *tx)
		.await {
			if crates.len()==0 {
				return Err(CrateFilterErr::NotFound);
			}
			self.result = Some(crates);
		} else {
			return Err(CrateFilterErr::NotFound);
		}
		Ok(())
	}

	async fn fetch_using_addr(&mut self) -> Result<(), CrateFilterErr> {
		let mut tx = db::db().await.begin().await.unwrap();
		let addrs = self.addr.clone().unwrap().into_iter()
			.map(|x| x.to_uuid()).collect::<Vec<String>>();
		if let Ok(crates) = sqlx::query_as::<_, Crate>(
			"
			SELECT DISTINCT ON (c.id) c.*
			FROM crate c
			JOIN crate_access ca ON ca.crate_id = c.id
			JOIN addr a ON ca.addr_id = a.id
			WHERE a.id = ANY($1::uuid[])
			AND ca.type = ANY($2)
			AND 
			CASE
				WHEN $3 != '' THEN c.id = $3::uuid
				WHEN $4 != '' THEN c.name = $4
				ELSE true
			END
			"
		)
		.bind(addrs)
		.bind(self.access_type.clone())
		.bind(self.crate_id.clone().unwrap_or("".to_string()))
		.bind(self.name.clone().unwrap_or("".to_string()))
		.fetch_all(&mut *tx)
		.await {
			if crates.len()==0 {
				return Err(CrateFilterErr::NotFound);
			}
			self.result = Some(crates);
		} else {
			return Err(CrateFilterErr::NotFound);
		}
		Ok(())
	}
}

// #[derive(Debug)]
// pub enum CrateFilterErr {
// 	BadData,
// 	NotFound,
// 	Multiple,
// 	Unauthorize,
// }

// impl Error for CrateFilterErr {}

// impl fmt::Display for CrateFilterErr {
//     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
//         match self {
//             CrateFilterErr::BadData => write!(f, "addr or name missing"),
//             CrateFilterErr::NotFound => write!(f, "no crate found"),
//             CrateFilterErr::Multiple => write!(f, "multiple create exists"),
//             CrateFilterErr::Unauthorize => write!(f, "no crate found with this access"),
//         }
//     }
// }