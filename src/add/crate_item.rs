use std::{error::Error, fmt};
use salvo::http::{StatusCode};
use salvo::prelude::{handler, Depot, Response, Request};
use secp256k1::PublicKey;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use sqlx::types::{Uuid};
use sqlx::FromRow;
use crate::{
	AddCrateItemReq, write_file, CrateItemStorage,
	CrateItem, CrateItemRes, AccessType, SpecialAddr,
	read_file, verify_hash,
	comn_addr::ComnAddr, db::{db, get_config},
	read::{
		crate_item::CrateItemFilter,
		crates::CrateFilter,
		addr::AddrFilter
	},
};
use std::convert::Infallible;
use std::time::Duration;
use futures_util::StreamExt;
use salvo::sse::{self, SseEvent};
use tokio::time::interval;
use tokio_stream::wrappers::IntervalStream;
use crate::print_current_db;

#[derive(FromRow, Serialize, Deserialize, Debug)]
pub struct AddCrateItem {
	pub crate_id: Uuid,
	pub addr: ComnAddr,
	pub item_path: String,
	pub media_type: String,
	pub sha2_hash: Vec<u8>,
	pub data: Option<Vec<u8>>,
	pub scope: String,
}

#[derive(Debug)]
pub enum AddCrateItemErr {
	InternalErr,
	PayloadLarge,
}

impl Error for AddCrateItemErr {}

impl fmt::Display for AddCrateItemErr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            AddCrateItemErr::InternalErr => write!(f, "internal error"),
            AddCrateItemErr::PayloadLarge => write!(f, "Payload is too large"),
        }
    }
}

impl AddCrateItem {
	pub async fn add(self) -> Result<Uuid, AddCrateItemErr> {
		let conf = get_config().await;

		if let Some(data) = self.data {
			if verify_hash(&*data, &*self.sha2_hash).await {
				let size_hectobyte = (data.len() / 100) as i32;
				if size_hectobyte > conf.data_size.max {
					return Err(AddCrateItemErr::PayloadLarge);
				}

				let mut tx = db().await.begin().await.unwrap();
				let id = Uuid::new_v4();
				let item_storage = if size_hectobyte > conf.data_size.max_db {
					if let Ok(_fname) =
						write_file(&format!("{}/{}_{}", conf.chunks_location, id, 0), &data).await
					{
						sqlx::query("INSERT INTO crate_item_chunk(id, crate_item_id, sha2_hash, size_hectobyte)
							VALUES(0, $1, $2, $3)").bind(id)
						.bind(self.sha2_hash).bind(size_hectobyte)
						.fetch_optional(&mut *tx).await.unwrap();
						Some(CrateItemStorage::File)
					} else {
						return Err(AddCrateItemErr::InternalErr);
						None
					}
				} else {
					None
				};

				let rr = match self.media_type.split_once('/').unwrap() {
					("application", "json") => {
						let tdata = if item_storage.is_some() {
							None
						} else {
							Some(String::from_utf8(data).unwrap_or_default())
						};
						sqlx::query_scalar::<_, Uuid>(
							"INSERT INTO
						crate_item(id, crate_id, item_path, data_json, type_id, size_hectobyte, item_storage, complete, added_by, scope_id)
							VALUES($1, $2, $3, $4::json, (SELECT id from item_type where media_type = $5),
							$6, $7, true, $8::uuid, (SELECT id from scope where scope_type = $9)) RETURNING id",
						)
						.bind(id)
						.bind(self.crate_id)
						.bind(self.item_path)
						.bind(tdata)
						.bind(self.media_type)
						.bind(size_hectobyte)
						.bind(item_storage.unwrap_or(CrateItemStorage::Json))
						.bind(self.addr.to_uuid())
						.bind(self.scope)
						.fetch_one(&mut *tx)
						.await
						.unwrap()
					}

					("text", ..) => {
						let tdata = if item_storage.is_some() {
							None
						} else {
							Some(String::from_utf8(data).unwrap_or_default())
						};
						sqlx::query_scalar::<_, Uuid>(
							"INSERT INTO
						crate_item(id, crate_id, item_path, data_text, type_id, size_hectobyte, item_storage, complete, added_by, scope_id)
							VALUES($1, $2, $3, $4, (SELECT id from item_type where media_type = $5),
							$6, $7, true, $8::uuid, (SELECT id from scope where scope_type = $9)) RETURNING id",
						)
						.bind(id)
						.bind(self.crate_id)
						.bind(self.item_path)
						.bind(tdata)
						.bind(self.media_type)
						.bind(size_hectobyte)
						.bind(item_storage.unwrap_or(CrateItemStorage::Text))
						.bind(self.addr.to_uuid())
						.bind(self.scope)
						.fetch_one(&mut *tx)
						.await
						.unwrap()
					}

					(_, _) => {
						let tdata = if item_storage.is_some() {
							None
						} else {
							Some(data)
						};
						sqlx::query_scalar::<_, Uuid>(
							"INSERT INTO
						crate_item(id, crate_id, item_path, data_bytes, type_id, size_hectobyte, item_storage, complete, added_by, scope_id)
							VALUES($1, $2, $3, $4, (SELECT id from item_type where media_type = $5),
							$6, $7, true, $8::uuid, (SELECT id from scope where scope_type = $9)) RETURNING id",
						)
						.bind(id)
						.bind(self.crate_id)
						.bind(self.item_path)
						.bind(tdata)
						.bind(self.media_type)
						.bind(size_hectobyte)
						.bind(item_storage.unwrap_or(CrateItemStorage::Bytes))
						.bind(self.addr.to_uuid())
						.bind(self.scope)
						.fetch_one(&mut *tx)
						.await
						.unwrap()
					}
				};
				tx.commit().await.unwrap();
				return Ok(rr);
			} else {
				return Err(AddCrateItemErr::InternalErr);
			}
		} else {
			let rr = sqlx::query_scalar::<_, Uuid>(
				"INSERT INTO
						crate_item(crate_id, item_path, data_bytes, media_type, size_hectobyte, item_storage, added_by, scope_id)
							VALUES($1, $2, $3, $4, 0, 'File', $5::uuid, (SELECT id from scope where scope_type = $6)) RETURNING id",
			)
			.bind(self.crate_id)
			.bind(self.item_path)
			.bind(self.sha2_hash)
			.bind(self.media_type)
			.bind(self.addr.to_uuid())
			.bind(self.scope)
			.fetch_one(db().await)
			.await
			.unwrap();
			return Ok(rr);
		}
	}
}