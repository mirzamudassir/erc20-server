mod common;
use common::{make_auth_header};
use comn_broker::{
	comn_addr::ComnAddr,
	AddCrateItemReq, CrateItem, CrateItemRes,
};


use salvo::http::header::AUTHORIZATION;
use salvo::prelude::*;
use salvo::test::{ResponseExt, TestClient};

use sha2::{Digest, Sha256};
use sqlx::PgPool;

use uuid::Uuid;

use comn_broker::read::{
	crates::{CrateOwnerFilter},
};


#[sqlx::test(fixtures("crate_item"), migrator = "comn_broker::MIGRATOR")]
async fn test_get_crate_item(_pool: PgPool) -> sqlx::Result<()> {
	common::setup(_pool).await;

	let mut res = TestClient::get(format!(
		"http://{}/item?id=a0000000000000000000000000000000",
		&std::env::var("BIND_ADDR").unwrap()
	))
	.add_header(
		AUTHORIZATION,
		&make_auth_header("Key1", "origin.com", "crate_read", 60 * 60 * 24 * 30, 0),
		true,
	)
	.send(comn_broker::route())
	.await;

	assert_eq!(res.status_code.unwrap(), StatusCode::OK);
	let item = res.take_json::<CrateItem>().await.unwrap();

	assert_eq!(
		item.data_text.unwrap(),
		"this is text string 1 for test_crate\\nyes it is!\\nb1 anon read, 0a owner"
	);

	let res_fail = TestClient::get(format!(
		"http://{}/item?id=b0000000000000000000000000000000",
		&std::env::var("BIND_ADDR").unwrap()
	))
	.send(comn_broker::route())
	.await;

	assert_eq!(res_fail.status_code.unwrap(), StatusCode::NOT_FOUND);

	let mut res_anon = TestClient::get(format!(
		"http://{}/item?id=e0000000000000000000000000000000",
		&std::env::var("BIND_ADDR").unwrap()
	))
	.send(comn_broker::route())
	.await;
	assert_eq!(res_anon.status_code.unwrap(), StatusCode::OK);
	let item_anon = res_anon.take_json::<CrateItem>().await.unwrap();
	assert_eq!(
		item_anon.data_text.unwrap(),
		"this is anon text string for test_crate\\nyes it is!\\nb1 anon read, 0a owner"
	);

	let mut res_pub_authed = TestClient::get(format!(
		"http://{}/item?id=e0000000000000000000000000000000",
		&std::env::var("BIND_ADDR").unwrap()
	))
	.add_header(
		AUTHORIZATION,
		&make_auth_header("Key2", "origin.com", "crate_read", 60 * 60 * 24 * 30, 0),
		true,
	)
	.send(comn_broker::route())
	.await;

	assert_eq!(res_pub_authed.status_code.unwrap(), StatusCode::OK);
	let item_pub_authed = res_pub_authed.take_json::<CrateItem>().await.unwrap();
	assert_eq!(
		item_pub_authed.data_text.unwrap(),
		"this is anon text string for test_crate\\nyes it is!\\nb1 anon read, 0a owner"
	);

	let mut res_pub_authed = TestClient::get(format!(
		"http://{}/item?id=d0000000000000000000000000000000",
		&std::env::var("BIND_ADDR").unwrap()
	))
	.add_header(
		AUTHORIZATION,
		&make_auth_header("Key2", "comn.opus.ai", "crate_read", 60 * 60 * 24 * 30, 0),
		true,
	)
	.send(comn_broker::route())
	.await;

	assert_eq!(res_pub_authed.status_code.unwrap(), StatusCode::OK);
	let item_pub_authed = res_pub_authed.take_json::<CrateItem>().await.unwrap();
	assert_eq!(
		item_pub_authed.data_text.unwrap(),
		"this is a text string for rest_cucket\\nyes it is!\\nb2 0b reader"
	);

	Ok(())
}

#[sqlx::test(fixtures("crate_item"), migrator = "comn_broker::MIGRATOR")]
async fn test_list_crate(_pool: PgPool) -> sqlx::Result<()> {
	common::setup(_pool).await;

	let mut res = TestClient::get(format!(
		"http://{}/crate/list?id=10000000000000000000000000000000",
		&std::env::var("BIND_ADDR").unwrap()
	))
	.add_header(
		AUTHORIZATION,
		&make_auth_header("Key1", "comn.opus.ai", "crate_read", 60 * 60 * 24 * 30, 0),
		true,
	)
	.send(comn_broker::route())
	.await;

	assert_eq!(res.status_code.unwrap(), StatusCode::OK);
	let resp = res.take_json::<Vec<CrateItemRes>>().await.unwrap();
	assert_eq!(resp.len(), 2);

	let mut res = TestClient::get(format!(
		"http://{}/crate/list?id=20000000000000000000000000000000",
		&std::env::var("BIND_ADDR").unwrap()
	))
	.add_header(
		AUTHORIZATION,
		&make_auth_header("Key2", "comn.opus.ai", "crate_read", 60 * 60 * 24 * 30, 0),
		true,
	)
	.send(comn_broker::route())
	.await;

	assert_eq!(res.status_code.unwrap(), StatusCode::OK);
	let resp = res.take_json::<Vec<CrateItemRes>>().await.unwrap();
	assert_eq!(resp.len(), 2);


	// TO-DO add again after completing crate_filter
	// wouldn't be able to read because write access is avilable for registered
	// let mut res3 = TestClient::get(format!(
	// 	"http://{}/crate/list?crate_ids=20000000000000000000000000000000",
	// 	&std::env::var("BIND_ADDR").unwrap()
	// ))
	// .add_header(
	// 	AUTHORIZATION,
	// 	&make_auth_header("randomKey", "comn.opus.ai", "crate_read", 60 * 60 * 24 * 30, 0),
	// 	true,
	// )
	// .send(comn_broker::route())
	// .await;
	// assert_eq!(res3.status_code.unwrap(), StatusCode::UNAUTHORIZED);

	// would be able to read because public has read access
	let res4 = TestClient::get(format!(
		"http://{}/crate/list?id=10000000000000000000000000000000",
		&std::env::var("BIND_ADDR").unwrap()
	))
	.add_header(
		AUTHORIZATION,
		&make_auth_header("randomKey", "comn.opus.ai", "crate_read", 60 * 60 * 24 * 30, 0),
		true,
	)
	.send(comn_broker::route())
	.await;
	assert_eq!(res4.status_code.unwrap(), StatusCode::OK);

	Ok(())
}

// TO-DO complete it
#[sqlx::test(fixtures("crate_item"), migrator = "comn_broker::MIGRATOR")]
async fn test_list_crate_stream(_pool: PgPool) -> sqlx::Result<()> {
	common::setup(_pool).await;

	let res = TestClient::get(format!(
		"http://{}/crate/list/stream?id=10000000000000000000000000000000",
		&std::env::var("BIND_ADDR").unwrap()
	))
	.add_header(
		AUTHORIZATION,
		&make_auth_header("Key1", "comn.opus.ai", "crate_read", 60 * 60 * 24 * 30, 0),
		true,
	)
	.send(comn_broker::route())
	.await;
	println!("res {:?}", res);

	Ok(())
}

#[sqlx::test(fixtures("crate_write"), migrator = "comn_broker::MIGRATOR")]
async fn test_add_crate_item(_pool: PgPool) -> sqlx::Result<()> {
	common::setup(_pool).await;
	{
		let data =
			"this is text string 1 for test_crate\\nyes it is!\\nb1 anon read, 0a owner".as_bytes();
		let mut hasher = Sha256::new();
		hasher.update(data);
		let sha2_hash = hasher.finalize().as_slice().to_vec();
		let add_item_req = AddCrateItemReq {
			crate_id: "10000000000000000000000000000000".to_string(),
			addr: ComnAddr::new("≈a").unwrap(),
			item_path: "/text_file".to_string(),
			media_type: "text/plain".to_string(),
			data: Some(data.into()),
			sha2_hash: sha2_hash,
		};

		let mut res = TestClient::post(format!(
			"http://{}/item",
			&std::env::var("BIND_ADDR").unwrap()
		))
		.add_header(
			AUTHORIZATION,
			&make_auth_header(
				"NewKey",
				"comn.opus.ai",
				"crate_write,storage",
				60 * 60 * 24 * 30,
				0,
			),
			true,
		)
		.json(&add_item_req)
		.send(comn_broker::route())
		.await;

		assert_eq!(res.status_code.unwrap(), StatusCode::OK);
		let item = res.take_json::<Uuid>().await.unwrap();

		let url = format!(
			"http://{}/item?id={}",
			&std::env::var("BIND_ADDR").unwrap(),
			item
		);
		let mut res_pub_authed = TestClient::get(url)
			.add_header(
				AUTHORIZATION,
				&make_auth_header("NewKey", "comn.opus.ai", "crate_read", 60 * 60 * 24 * 30, 0),
				true,
			)
			.send(comn_broker::route())
			.await;

		assert_eq!(res_pub_authed.status_code.unwrap(), StatusCode::OK);
		let item_pub_authed = res_pub_authed.take_json::<CrateItem>().await.unwrap();
		assert_eq!(
			item_pub_authed.data_text.unwrap(),
			String::from_utf8(data.to_vec()).unwrap()
		);
	}
	{
		use rand::prelude::*;

		let mut data: [u8; 3_000] = [1; 3_000];
		rand::thread_rng().fill(&mut data[..]);
		let mut hasher = Sha256::new();
		hasher.update(&data);
		let sha2_hash = hasher.finalize().as_slice().to_vec();
		let add_item_req = AddCrateItemReq {
			crate_id: "10000000000000000000000000000000".to_string(),
			addr: ComnAddr::new("≈a").unwrap(),
			item_path: "/3k_binary".to_string(),
			media_type: "image/jpeg".to_string(),
			data: Some(data.into()),
			sha2_hash: sha2_hash,
		};

		let mut res = TestClient::post(format!(
			"http://{}/item",
			&std::env::var("BIND_ADDR").unwrap()
		))
		.add_header(
			AUTHORIZATION,
			&make_auth_header(
				"NewKey",
				"comn.opus.ai",
				"crate_write,storage",
				60 * 60 * 24 * 30,
				0,
			),
			true,
		)
		.json(&add_item_req)
		.send(comn_broker::route())
		.await;

		assert_eq!(res.status_code.unwrap(), StatusCode::OK);
		let item = res.take_json::<Uuid>().await.unwrap();

		let url = format!(
			"http://{}/item?id={}",
			&std::env::var("BIND_ADDR").unwrap(),
			item
		);
		let mut res_pub_authed = TestClient::get(url)
			.add_header(
				AUTHORIZATION,
				&make_auth_header("NewKey", "comn.opus.ai", "crate_read", 60 * 60 * 24 * 30, 0),
				true,
			)
			.send(comn_broker::route())
			.await;

		assert_eq!(res_pub_authed.status_code.unwrap(), StatusCode::OK);
		let item_pub_authed = res_pub_authed.take_json::<CrateItem>().await.unwrap();
		// println!("{:?}", item_pub_authed);
		assert_eq!(item_pub_authed.data_file.unwrap(), data.to_vec());
	}

	Ok(())
}