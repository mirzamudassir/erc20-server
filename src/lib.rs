pub mod auth_token;
pub mod comn_addr;
pub mod db;
pub mod utils;
pub mod read;
pub mod handlers;
pub mod update;
pub mod add;

use auth_token::{check_auth, force_auth, protected};
use chrono::{DateTime, Utc};
use comn_addr::ComnAddr;
use db::{db};
use salvo::http::{header};
use salvo::prelude::{
	handler, Listener, OpenApi, Response, Router, Server, SwaggerUi, TcpListener,
};
use serde::{Deserialize, Serialize};
use sqlx::migrate::Migrator;
use sqlx::types::{Uuid};
use sqlx::FromRow;
use sqlx::postgres::{PgTypeInfo, PgHasArrayType};
use crate::read::{crates::{CrateOwnerFilter}};
use crate::handlers::{coin, addr, crates, crate_item, stripe};
use sha2::{Sha256, Digest};
// use regex_lite::Regex;
// use std::{error::Error, fmt};
// use mediatype::MediaType;

pub static MIGRATOR: Migrator = sqlx::migrate!("db/migrations");

pub async fn print_current_db() {
	println!("{}", sqlx::query_scalar::<_, String>("Select current_database();").fetch_one(db::db().await).await.unwrap());
}

pub enum SpecialAddr {
	Public,
	Registered,
	Config,
	ComnCoin,
}

impl SpecialAddr {
    fn value(&self) -> ComnAddr {
        match *self {
            SpecialAddr::Public => ComnAddr::from_uuid("ffffffffffffffffffffffffffffffff").unwrap(),
            SpecialAddr::Registered => ComnAddr::from_uuid("fffffffffffffffffffffffffffffffe").unwrap(),
            SpecialAddr::Config => ComnAddr::from_uuid("00000000000000000000000000000000").unwrap(),
            SpecialAddr::ComnCoin => ComnAddr::from_uuid("000000000000000000000000000000cc").unwrap(),
        }
    }
}

#[derive(sqlx::Type, Serialize, Deserialize, Debug, PartialEq, Clone)]
#[sqlx(type_name = "ACCESS_TYPE")]
#[sqlx(rename_all = "lowercase")]
pub enum AccessType {
    Owner,
    Admin,
    Editor,
    Reader,
    Writer,
}

impl PgHasArrayType for AccessType {
    fn array_type_info() -> PgTypeInfo {
        PgTypeInfo::with_name("_ACCESS_TYPE")
    }
}

#[derive(FromRow, Serialize, Deserialize, Debug)]
pub struct AddCrateReq {
	pub name: String,
	pub comment: String,
	pub addr: ComnAddr,
	pub expires: Option<DateTime<Utc>>,
}

#[derive(FromRow, Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct Crate {
	pub id: Uuid,
	pub name: String,
	pub comment: Option<String>,
	pub expires: Option<DateTime<Utc>>,
	pub created: DateTime<Utc>,
}

#[derive(FromRow, Serialize, Deserialize, Debug)]
pub struct CrateAccess {
	pub crate_id: Uuid,
	pub addr_id: Uuid,
	pub r#type: AccessType,
	pub expires: Option<DateTime<Utc>>,
	pub created: DateTime<Utc>,
}

#[derive(FromRow, Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct CrateItem {
	pub id: Uuid,
	pub added_by: Uuid,
	pub crate_id: Uuid,
	pub scope: String,
	pub item_path: String,
	pub item_storage: CrateItemStorage,
	pub media_type: String,
	pub data_json: Option<sqlx::types::Json<serde_json::Value>>,
	pub data_text: Option<String>,
	pub data_bytes: Option<Vec<u8>>,
	pub data_file: Option<Vec<u8>>,
	pub chunk_count: i16,
	pub size_hectobyte: i32,
	pub complete: bool,
	pub expires: Option<DateTime<Utc>>,
	pub created: DateTime<Utc>,
	pub updated: DateTime<Utc>,
}

#[derive(FromRow, Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct CrateItemRes {
	pub id: Uuid,
	pub added_by: ComnAddr,
	pub crate_id: Uuid,
	pub scope: String,
	pub item_path: String,
	pub item_storage: CrateItemStorage,
	pub media_type: String,
	pub data_json: Option<sqlx::types::Json<serde_json::Value>>,
	pub data_text: Option<String>,
	pub data_bytes: Option<Vec<u8>>,
	pub data_file: Option<Vec<u8>>,
	pub chunk_count: i16,
	pub size_hectobyte: i32,
	pub complete: bool,
	pub expires: Option<DateTime<Utc>>,
	pub created: DateTime<Utc>,
	pub updated: DateTime<Utc>,
}

impl From<CrateItem> for CrateItemRes {
  fn from(a: CrateItem) -> Self {
    Self {
			id: a.id,
			added_by: ComnAddr::from_uuid(&a.added_by.to_string()).unwrap(),
			crate_id: a.crate_id,
			scope: a.scope,
			item_path: a.item_path,
			item_storage: a.item_storage,
			media_type: a.media_type,
			data_json: a.data_json,
			data_text: a.data_text,
			data_bytes: a.data_bytes,
			data_file: a.data_file,
			chunk_count: a.chunk_count,
			size_hectobyte: a.size_hectobyte,
			complete: a.complete,
			expires: a.expires,
			created: a.created,
			updated: a.updated,
    }
  }
}

#[derive(FromRow, Serialize, Deserialize, Debug)]
pub struct AddCrateItemReq {
	pub crate_id: String,
	pub addr: ComnAddr,
	pub item_path: String,
	pub media_type: String,
	pub data: Option<Vec<u8>>,
	pub sha2_hash: Vec<u8>,
}

#[derive(sqlx::Type, Serialize, Deserialize, Debug, PartialEq, Copy, Clone)]
pub enum CrateItemStorage {
	Json,
	Text,
	Bytes,
	File,
}

pub async fn write_file(fname: &str, data: &[u8]) -> Result<String, &'static str> {
	use tokio::fs::File;
	use tokio::io::AsyncWriteExt;
	if let Ok(mut file) = File::create(fname).await {
		if let Ok(_res) = file.write_all(data).await {
			return Ok("Done".to_string());
		}
	}
	panic!("FileWriteFailure");
}

async fn read_file(fname: &str) -> Result<Vec<u8>, &'static str> {
	use tokio::fs::File;
	use tokio::io::AsyncReadExt;
	if let Ok(mut file) = File::open(fname).await {
		let mut contents = vec![];
		if let Ok(_r) = file.read_to_end(&mut contents).await {
			return Ok(contents);
		}
	}
	panic!("FileReadFailure");
}

async fn verify_hash(data: &[u8], hash: &[u8]) -> bool {
	let mut hasher = Sha256::new();
	hasher.update(data);
	hasher.finalize().as_slice().to_vec() == hash
}

async fn _add_crate(crate_req: AddCrateReq) -> Crate {
	let mut tx = db().await.begin().await.unwrap();

	let rr = sqlx::query_as::<_, Crate>(
		"INSERT INTO crate(name, comment, expires) VALUES($1, $2, $3) RETURNING *",
	)
	.bind(crate_req.name)
	.bind(crate_req.comment)
	.bind(crate_req.expires)
	.fetch_one(&mut *tx)
	.await
	.unwrap();
	sqlx::query(
		"INSERT INTO crate_access(crate_id, addr_id, type)
			VALUES ($1::uuid, $2::uuid, 'owner') RETURNING crate_id",
	)
	.bind(rr.id)
	.bind(crate_req.addr.to_uuid())
	.fetch_optional(&mut *tx)
	.await
	.unwrap();
	tx.commit().await.unwrap();

	rr
}

// enum MatchType {
// 	Host,
// 	Header,
// 	Path,
// }
// struct Match {
// 	match_type: MatchType,
// 	value: Regex,
// }

// enum ActionType {
// 	Rewrite,
// }
// struct Action {
// 	action_type: ActionType,
// 	value: String,
// }

#[handler]
pub async fn cors_handler(res: &mut Response) {
	// let origin = req.headers().get(&header::ORIGIN);
	let mut headers = header::HeaderMap::new();
	headers.insert(header::ACCESS_CONTROL_ALLOW_ORIGIN, "*".parse().unwrap());
	headers.insert(header::ACCESS_CONTROL_ALLOW_METHODS, "*".parse().unwrap());
	headers.insert(header::ACCESS_CONTROL_ALLOW_HEADERS, "*".parse().unwrap());
	res.headers_mut().extend(headers);
}

pub fn route() -> salvo::Router {

	let router = Router::with_hoop(cors_handler)
		.push(Router::with_path("<**rest_path>").options(handler::empty()))
		.push(Router::with_path("/").hoop(check_auth).get(crates::list_crates))
		.push(Router::with_path("/").hoop(force_auth).post(crates::add_crate))
		.push(Router::with_path("addr").hoop(check_auth).get(addr::get_addr))
		.push(Router::with_path("key").get(addr::get_keys))
		.push(
			Router::with_path("key")
				.hoop(force_auth)
				.post(addr::register_key)
		)
		.push(
			Router::with_path("item")
				.hoop(check_auth)
				.get(crate_item::get_crate_item),
		)
		.push(
			Router::with_path("crate")
				.hoop(force_auth)
				.get(crates::get_crate)
				.post(crates::add_crate)
				.push(
					Router::with_path("list")
						.get(crate_item::list_crate)
						.push(
							Router::with_path("stream")
								.get(crate_item::list_crate_stream),
						)
				)
				.push(Router::with_path("access").post(crates::change_crate_access)),
		)
		.push(
			Router::with_path("item")
				.hoop(force_auth)
				.post(crate_item::add_crate_item), // .put(update_crate_item)
		)
		.push(
			Router::with_path("comn")
				.hoop(force_auth)
				.get(coin::get_comn_coins)
		)
		.push(
			Router::with_path("comn")
				.push(
					Router::with_path("transaction")
						.hoop(protected)
						.post(coin::transaction),
				)
		)
		.push(Router::with_path("stripe_webhook").hoop(stripe::check_sign).post(stripe::stripe_webhook));
	let doc = OpenApi::new("api", "0.0.1").merge_router(&router);
	router
		.push(doc.into_router("/api-doc/openapi.json"))
		.push(SwaggerUi::new("/api-doc/openapi.json").into_router("swagger-ui"))
}

pub async fn app() {
	utils::default_env("DATABASE_URL", "postgres://comn:password@localhost/comn");
	utils::default_env("BIND_ADDR", "127.0.0.1:5800");
}

pub async fn serve() {
	let _ = MIGRATOR.run(db::db().await).await;
	let acceptor = TcpListener::new(&std::env::var("BIND_ADDR").unwrap())
		.bind()
		.await;
	Server::new(acceptor).serve(route()).await;
}
