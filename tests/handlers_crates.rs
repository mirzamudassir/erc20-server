mod common;
use common::{make_auth_header};
use comn_broker::{
	comn_addr::ComnAddr, AddCrateReq, Crate, AccessType,
};
use comn_broker::handlers::{
	crates::{CrateAccessReq}
};
use salvo::http::header::AUTHORIZATION;
use salvo::prelude::*;
use salvo::test::{ResponseExt, TestClient};
use sqlx::PgPool;
use std::time::SystemTime;
use uuid::Uuid;

#[sqlx::test(fixtures("addr_key"), migrator = "comn_broker::MIGRATOR")]
async fn test_add_crate(_pool: PgPool) -> sqlx::Result<()> {
	common::setup(_pool).await;

	let crate_req = AddCrateReq {
		name: "test_crate".to_string(),
		comment: "".to_string(),
		addr: ComnAddr::new("≈a").unwrap(),
		expires: Some(SystemTime::now().into()),
	};
	let mut res = TestClient::post(format!(
		"http://{}/crate",
		&std::env::var("BIND_ADDR").unwrap()
	))
	.add_header(
		AUTHORIZATION,
		&make_auth_header(
			"NewKey",
			"comn.opus.ai",
			"crate_write",
			60 * 60 * 24 * 30,
			0,
		),
		true,
	)
	.json(&crate_req)
	.send(comn_broker::route())
	.await;

	assert_eq!(res.status_code.unwrap(), StatusCode::OK);
	let resp = res.take_json::<Crate>().await.unwrap();
	assert_eq!(resp.name, crate_req.name);

	Ok(())
}

#[sqlx::test(fixtures("crate"), migrator = "comn_broker::MIGRATOR")]
async fn test_get_crate(_pool: PgPool) -> sqlx::Result<()> {
	common::setup(_pool).await;

	// getting crate with id
	let mut res = TestClient::get(format!(
		"http://{}/crate?id=10000000000000000000000000000000",
		&std::env::var("BIND_ADDR").unwrap()
	))
	.add_header(
		AUTHORIZATION,
		&make_auth_header("NewKey", "comn.opus.ai", "crate_get", 60 * 60 * 24 * 30, 0),
		true,
	)
	.send(comn_broker::route())
	.await;

	assert_eq!(res.status_code.unwrap(), StatusCode::OK);
	let resp = res.take_json::<Crate>().await.unwrap();
	assert_eq!(resp.name, "test_crate");

	//doesn't have access
	let res = TestClient::get(format!(
		"http://{}/crate?id=30000000000000000000000000000000",
		&std::env::var("BIND_ADDR").unwrap()
	))
	.add_header(
		AUTHORIZATION,
		&make_auth_header("NewKey", "comn.opus.ai", "crate_get", 60 * 60 * 24 * 30, 0),
		true,
	)
	.send(comn_broker::route())
	.await;
	assert_eq!(res.status_code.unwrap(), StatusCode::BAD_REQUEST);

	//getting crate from name and owner
	let mut res = TestClient::get(format!(
		"http://{}/crate?name=test_crate&addr=≈A&access=Owner",
		&std::env::var("BIND_ADDR").unwrap()
	))
	.add_header(
		AUTHORIZATION,
		&make_auth_header("NewKey", "comn.opus.ai", "crate_get", 60 * 60 * 24 * 30, 0),
		true,
	)
	.send(comn_broker::route())
	.await;

	assert_eq!(res.status_code.unwrap(), StatusCode::OK);
	let resp = res.take_json::<Crate>().await.unwrap();
	println!("resq {:?}", resp);
	assert_eq!(resp.id, Uuid::parse_str("20000000-0000-0000-0000-000000000000").unwrap());

	//getting crate with registered addr read access
	let mut res = TestClient::get(format!(
		"http://{}/crate?id=40000000000000000000000000000000",
		&std::env::var("BIND_ADDR").unwrap()
	))
	.add_header(
		AUTHORIZATION,
		&make_auth_header("NewKey", "comn.opus.ai", "crate_get", 60 * 60 * 24 * 30, 0),
		true,
	)
	.send(comn_broker::route())
	.await;

	assert_eq!(res.status_code.unwrap(), StatusCode::OK);
	let resp = res.take_json::<Crate>().await.unwrap();
	assert_eq!(resp.name, "test_crate_4");

	// crate doesn't exists
	let res = TestClient::get(format!(
		"http://{}/crate?name=test_crate_5&addr=≈A&access=Owner",
		&std::env::var("BIND_ADDR").unwrap()
	))
	.add_header(
		AUTHORIZATION,
		&make_auth_header("NewKey", "comn.opus.ai", "crate_get", 60 * 60 * 24 * 30, 0),
		true,
	)
	.send(comn_broker::route())
	.await;

	assert_eq!(res.status_code.unwrap(), StatusCode::NOT_FOUND);

	Ok(())
}


#[sqlx::test(fixtures("crate_item"), migrator = "comn_broker::MIGRATOR")]
async fn test_list_crates(_pool: PgPool) -> sqlx::Result<()> {
	common::setup(_pool).await;

	let mut res = TestClient::get(format!("http://{}/", &std::env::var("BIND_ADDR").unwrap()))
		.add_header(
			AUTHORIZATION,
			&make_auth_header("Key1", "comn.opus.ai", "crate_read", 60 * 60 * 24 * 30, 0),
			true,
		)
		.send(comn_broker::route())
		.await;

	assert_eq!(res.status_code.unwrap(), StatusCode::OK);
	let resp = res.take_json::<Vec<Crate>>().await.unwrap();
	println!("{:?}", resp);
	assert_eq!(resp.len(), 2);

	let mut res = TestClient::get(format!("http://{}/", &std::env::var("BIND_ADDR").unwrap()))
		.add_header(
			AUTHORIZATION,
			&make_auth_header("Key2", "comn.opus.ai", "crate_read", 60 * 60 * 24 * 30, 0),
			true,
		)
		.send(comn_broker::route())
		.await;

	assert_eq!(res.status_code.unwrap(), StatusCode::OK);
	let resp = res.take_json::<Vec<Crate>>().await.unwrap();
	assert_eq!(resp.len(), 1);

	Ok(())
}


#[sqlx::test(fixtures("crate_access"), migrator = "comn_broker::MIGRATOR")]
async fn test_crate_access(_pool: PgPool) -> sqlx::Result<()> {
	common::setup(_pool).await;
	let mut access_req = CrateAccessReq {
		crate_id: "10000000000000000000000000000000".to_string(),
		access_to_addr: "≈7ZZZZZZZZZZZZZZZZZZZZZZZZY".to_string(), // registered addr
		give_access: true,
		access_type: AccessType::Writer,
		expires: None,
	};
	println!("access req{:?}", access_req);
	// adding write access
	let res = TestClient::post(format!(
		"http://{}/crate/access",
		&std::env::var("BIND_ADDR").unwrap()
	))
	.add_header(
		AUTHORIZATION,
		&make_auth_header(
			"NewKey",
			"comn.opus.ai",
			"change_crate_access",
			60 * 60 * 24 * 30,
			0,
		),
		true,
	)
	.json(&access_req)
	.send(comn_broker::route())
	.await;
	assert_eq!(res.status_code.unwrap(), StatusCode::OK);


	// failing to remove owner
	access_req.access_type = AccessType::Owner;
	access_req.give_access = false;
	access_req.access_to_addr = ComnAddr::new("≈a").unwrap().to_string();
	let res2 = TestClient::post(format!(
		"http://{}/crate/access",
		&std::env::var("BIND_ADDR").unwrap()
	))
	.add_header(
		AUTHORIZATION,
		&make_auth_header(
			"NewKey",
			"comn.opus.ai",
			"change_crate_access",
			60 * 60 * 24 * 30,
			0,
		),
		true,
	)
	.json(&access_req)
	.send(comn_broker::route())
	.await;
	assert_eq!(res2.status_code.unwrap(), StatusCode::BAD_REQUEST);


	// adding owner access
	access_req.give_access = true;
	access_req.access_to_addr = ComnAddr::new("≈7ZZZZZZZZZZZZZZZZZZZZZZZZY").unwrap().to_string();
	let res3 = TestClient::post(format!(
		"http://{}/crate/access",
		&std::env::var("BIND_ADDR").unwrap()
	))
	.add_header(
		AUTHORIZATION,
		&make_auth_header(
			"NewKey",
			"comn.opus.ai",
			"change_crate_access",
			60 * 60 * 24 * 30,
			0,
		),
		true,
	)
	.json(&access_req)
	.send(comn_broker::route())
	.await;
	assert_eq!(res3.status_code.unwrap(), StatusCode::OK);


	// successfully removing owner access
	access_req.access_type = AccessType::Owner;
	access_req.give_access = false;
	let res4 = TestClient::post(format!(
		"http://{}/crate/access",
		&std::env::var("BIND_ADDR").unwrap()
	))
	.add_header(
		AUTHORIZATION,
		&make_auth_header(
			"NewKey",
			"comn.opus.ai",
			"change_crate_access",
			60 * 60 * 24 * 30,
			0,
		),
		true,
	)
	.json(&access_req)
	.send(comn_broker::route())
	.await;
	assert_eq!(res4.status_code.unwrap(), StatusCode::OK);

	Ok(())
}