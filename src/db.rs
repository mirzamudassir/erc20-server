use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool};
use tokio::sync::OnceCell;

static mut DB: OnceCell<PgPool> = OnceCell::const_new();

#[inline]
pub async fn db() -> &'static PgPool {
	unsafe {
		DB.get_or_init(|| async {
			PgPool::connect(&std::env::var("DATABASE_URL").unwrap())
				.await
				.unwrap()
		})
		.await
	}
}

#[inline]
pub fn set_db(val: PgPool) {
	unsafe {
		DB.take();
		let _ = DB.set(val);
	}
}

#[derive(FromRow, Serialize, Deserialize, Debug, Clone)]
pub struct DataSize {
	pub max: i32,
	pub max_chunk: i32,
	pub max_db: i32,
}

#[derive(FromRow, Serialize, Deserialize, Debug, Clone)]
pub struct Config {
	pub host_names: Vec<String>,
	pub data_size: DataSize,
	pub chunks_location: String,
}

pub static CONFIG: OnceCell<Config> = OnceCell::const_new();

pub async fn get_config() -> &'static Config {
	CONFIG
		.get_or_init(|| async {
			let item = sqlx::query_scalar::<_, serde_json::Value>(
				"
			SELECT bi.data_json FROM crate_item bi
			JOIN crate b ON b.id = bi.crate_id
			JOIN crate_access ba ON ba.crate_id = b.id
			WHERE b.id = '00000000-0000-0000-0000-000000000000'
			and ba.addr_id = '00000000-0000-0000-0000-000000000000'
			and ba.type is not null and bi.item_path = '/Config'
			",
			)
			.fetch_one(db().await)
			.await;
			if item.is_ok() {
				serde_json::from_value::<Config>(item.unwrap()).unwrap()
			} else {
				panic!("cannot get config {:?}", item);
			}
		})
		.await
}
