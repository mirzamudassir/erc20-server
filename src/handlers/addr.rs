use chrono::{DateTime, Utc};
use salvo::http::{StatusCode};
use salvo::prelude::{handler, Depot, Response, Request};
use secp256k1::PublicKey;
use serde::{Deserialize, Serialize};
use sqlx::types::{Uuid};
use sqlx::FromRow;
use crate::{
	comn_addr::ComnAddr, db::{db},
	read::addr::AddrFilter,
};
use crate::print_current_db;


#[derive(FromRow, Serialize, Deserialize, Debug)]
pub struct AddrKey {
	pub id: Uuid,
	pub pub_key: Vec<u8>,
	pub key_type: KeyType,
	pub created: DateTime<Utc>,
}

#[derive(sqlx::Type, Serialize, Deserialize, PartialEq, Eq, Debug)]
#[sqlx(type_name = "addr_key_type")] // only for PostgreSQL to match a type definition
#[sqlx(rename_all = "lowercase")]
pub enum KeyType {
	Operation,
	Recovery,
}


#[derive(Serialize, Deserialize, Debug)]
struct AddrReq {
	addr: Option<ComnAddr>,
	name: Option<String>,
}

impl From<AddrReq> for AddrFilter {
  fn from(a: AddrReq) -> Self {
    Self {
      addr: a.addr,
      name: a.name,
      result: None,
      keys: None,
    }
  }
}

#[handler]
pub async fn get_addr(req: &mut Request, res: &mut Response, depot: &mut Depot) {
	//TO-DO impl for Vec<addr> to Vec<AddrRes> then do fetch_all instead of fetch_one
	print_current_db().await;
	let param = req.parse_queries::<AddrReq>().unwrap();
	let mut addr_filter = AddrFilter::from(param);
	if let Ok(pub_key) = depot.get::<PublicKey>("public_key") {
		addr_filter.keys = Some(vec!(*pub_key));
	}

	if let Ok(_) = addr_filter.init().await {
		res.render(serde_json::to_string(&addr_filter.to_res()).unwrap());
	} else {
		res.render(StatusCode::BAD_REQUEST);
	}
}

#[handler]
pub async fn get_keys(req: &mut Request, res: &mut Response) {
	// print_current_db().await;
	if let Some(param) = req.query::<&str>("addr") {
		if let Ok(addr) = ComnAddr::new(param) {
			let data = sqlx::query_as::<_, AddrKey>(
				"
				SELECT a.id, k.pub_key, ak.key_type, k.created
				FROM addr a JOIN addr_key ak ON ak.addr_id = a.id
				JOIN key k on k.id = ak.key_id WHERE a.id = $1::uuid
				",
			)
			.bind(addr.to_uuid())
			.fetch_all(db().await)
			.await
			.unwrap();
			res.render(serde_json::to_string(&data).unwrap());
		} else {
			res.render(StatusCode::BAD_REQUEST);
		}
	} else {
		res.render(StatusCode::BAD_REQUEST);
	}
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RegResp {
	pub addr: ComnAddr,
	pub key_id: Uuid,
}
#[derive(Serialize, Deserialize, Debug)]
pub struct RegisterKeyReq {
	pub name: Option<String>,
}
/// Registers a private key and returns the address (Also see RegProof)
///
/// Retruns http status code REQUEST_TIMEOUT if the signature is expired or is constructed in the future
/// Returns http status code NOT_ACCEPTABLE if the signature is invalid
/// On success, returns http status code OK and the address + key_id (See RegResp)
#[handler]
pub async fn register_key(req: &mut Request, res: &mut Response, depot: &mut Depot) {
	// print_current_db().await;
	let pub_key = depot.get::<PublicKey>("public_key").unwrap();
	let mut tx = db().await.begin().await.unwrap();
	let register_key_req = req.parse_json::<RegisterKeyReq>()
		.await
		.unwrap_or(
			RegisterKeyReq { name: None }
		);
	let key_id = sqlx::query_scalar::<_, Uuid>("INSERT INTO key(pub_key) VALUES($1) RETURNING id")
		.bind(pub_key.serialize())
		.fetch_one(&mut *tx)
		.await
		.unwrap();
	let addr_id = sqlx::query_scalar::<_, Uuid>("INSERT INTO addr (name) VALUES ($1) RETURNING id")
		.bind(register_key_req.name)
		.fetch_one(&mut *tx)
		.await
		.unwrap();
	sqlx::query(
		"INSERT INTO addr_key(addr_id, key_id, key_type) VALUES ($1::uuid, $2::uuid, 'operation')",
	)
	.bind(addr_id)
	.bind(key_id)
	.fetch_optional(&mut *tx)
	.await
	.unwrap();
	tx.commit().await.unwrap();
	let rr = RegResp {
		addr: ComnAddr::from_uuid(&addr_id.to_string()).unwrap(),
		key_id,
	};
	res.render(serde_json::to_string(&rr).unwrap());
}