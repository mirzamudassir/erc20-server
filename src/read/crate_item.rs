use std::{error::Error, fmt};
use serde::{Deserialize, Serialize};

use secp256k1::PublicKey;


use crate::{
	db, AccessType, SpecialAddr,
	ComnAddr, CrateItem,
	CrateItemRes
};
use super::crates::{CrateFilter};

#[derive(Debug)]
pub enum CrateItemErr {
	BadData,
	NotFound,
	CrateNotFound,
	NotUpdated,
}

impl Error for CrateItemErr {}

impl fmt::Display for CrateItemErr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            CrateItemErr::BadData => write!(f, "request is bad"),
            CrateItemErr::NotFound => write!(f, "no item found"),
            CrateItemErr::CrateNotFound => write!(f, "Crate doesn't exist or doesn't have access to it."),
            CrateItemErr::NotUpdated => write!(f, "no new item added."),
        }
    }
}

// access would be checked at the start of request. So in the case of sse if the user
// has access to crate in the start of the connection and access is updated in the middle 
// of the connection he will keep on getting updates for the connection duration
#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct CrateItemFilter {
	pub addr: Option<ComnAddr>,
	pub pub_key: Option<PublicKey>,
	pub crate_id: String,
	pub per_page: u16,
	pub page_no: u16,
	pub result: Option<Vec<CrateItem>>
}


impl CrateItemFilter {
	pub async fn init(&mut self) -> Result<(), CrateItemErr> {
		let mut crate_addr: Vec<ComnAddr> = vec!(
			SpecialAddr::Registered.value(),
			SpecialAddr::Public.value(),
		);
		if let Some(addr) = self.addr.clone() {
			crate_addr.push(addr);
		}
		let mut crate_result = CrateFilter {
			name: None,
			addr: Some(crate_addr),
			pub_key: self.pub_key,
			crate_id: Some(self.crate_id.clone()),
			access_type: vec!(
				AccessType::Owner,
			    AccessType::Admin,
			    AccessType::Editor,
			    AccessType::Reader
			),
			result: None,
		};
		if let Ok(_) = crate_result.init().await {
			self.fetch().await?;
		} else {
			return Err(CrateItemErr::CrateNotFound);
		}
		Ok(())
	}
	pub fn to_res(&mut self) -> Result<Vec<CrateItemRes>, CrateItemErr> {
		let r: Vec<CrateItemRes> = self.result.clone().unwrap().into_iter().map(|r| CrateItemRes::from(r)).collect();
		Ok(r)
	}
	pub async fn update_res(&mut self) -> Result<Vec<CrateItemRes>, CrateItemErr> {
		self.fetch().await?;
		self.to_res()
	}
	pub async fn next_page(&mut self) -> Result<(), CrateItemErr> {
		self.page_no += 1;
		self.fetch().await
	}
	async fn fetch(&mut self) -> Result<(), CrateItemErr> {
		if self.pub_key == None {
			self.fetch_using_addr().await?;
		} else {
			self.fetch_using_key().await?;
		}
		Ok(())
	}
	async fn fetch_using_key(&mut self) -> Result<(), CrateItemErr> {
		let mut tx = db::db().await.begin().await.unwrap();
		let offset = self.page_no * self.per_page;
		let crate_items = sqlx::query_as::<_, CrateItem>(
			"
			SELECT
			ci.*, it.media_type, NULL as data_file, s.scope_type as scope,
			(SELECT COUNT(*) FROM crate_item_chunk WHERE crate_item_id = ci.id)::SMALLINT AS chunk_count
			FROM crate_item ci
			JOIN item_type it ON it.id = ci.type_id
			JOIN scope s ON s.id = ci.scope_id
			JOIN crate c ON ci.crate_id = c.id
			WHERE c.id = $1::uuid
			ORDER BY ci.created DESC
			OFFSET $2
			LIMIT $3
			"
		)
		.bind(self.crate_id.clone())
		.bind(offset as i16)
		.bind(self.per_page as i16)
		.fetch_all(&mut *tx)
		.await
		.unwrap();
		if crate_items.len()==0 {
			return Err(CrateItemErr::NotFound);
		}
		self.result = Some(crate_items);
		Ok(())
	}

	async fn fetch_using_addr(&mut self) -> Result<(), CrateItemErr> {
		// let mut tx = db::db().await.begin().await.unwrap();
		// if let Ok(crates) = sqlx::query_as::<_, Crate>(
		// 	"
		// 	SELECT c.*
		// 	FROM crate c
		// 	JOIN crate_access ca ON ca.crate_id = c.id
		// 	JOIN addr a ON ca.addr_id = a.id
		// 	WHERE (a.id = $1::uuid OR a.id = $2)
		// 	AND ca.type = ANY($3)
		// 	AND c.id = $4
		// 	"
		// )
		// .bind(self.addr.clone().unwrap().to_uuid())
		// .bind(SpecialAddr::Registered.value().to_uuid())
		// .bind(self.access_type.clone())
		// .bind(self.crate_id.clone())
		// .fetch_all(&mut *tx)
		// .await {
		// 	if crates.len()==0 {
		// 		return Err(CrateItemErr::NotFound);
		// 	}
		// 	self.result = Some(crates);
		// } else {
		// 	return Err(CrateItemErr::NotFound);
		// }
		Ok(())
	}
}