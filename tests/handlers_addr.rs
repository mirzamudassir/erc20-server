mod common;
use common::{get_keys, make_auth_header};
use comn_broker::{
	auth_token::{AuthToken}, comn_addr::ComnAddr,
};
use comn_broker::handlers::{
	addr::{AddrKey, RegisterKeyReq, RegResp, KeyType}
};
use salvo::http::header::AUTHORIZATION;
use salvo::prelude::*;
use salvo::test::{ResponseExt, TestClient};
use secp256k1::PublicKey;
use sqlx::PgPool;
use comn_broker::read::{
	addr::AddrRes,
};


#[sqlx::test(migrator = "comn_broker::MIGRATOR")]
async fn test_register_key_without_body(_pool: PgPool) -> sqlx::Result<()> {
	common::setup(_pool).await;
	let (secret_key, public_key) = get_keys("NewKey");
	let mut res = TestClient::post(format!(
		"http://{}/key",
		&std::env::var("BIND_ADDR").unwrap()
	))
	.add_header(
		AUTHORIZATION,
		&make_auth_header("NewKey", "comn.opus.ai", "create_key", 15, 5),
		true,
	)
	.send(comn_broker::route())
	.await;

	let body = res.take_json::<RegResp>().await.unwrap();
	println!("body {:?}", body);
	// assert_eq!(res.status_code.unwrap(), StatusCode::OK);
	// println!("test_register_key {} ({})", body.addr, serde_json::to_string(&body).unwrap());

	let mut res2 = TestClient::get(format!(
		"http://{}/key?addr={}",
		&std::env::var("BIND_ADDR").unwrap(),
		body.addr
	))
	.send(comn_broker::route())
	.await;
	assert_eq!(res2.status_code.unwrap(), StatusCode::OK);
	let addrs = res2.take_json::<Vec<AddrKey>>().await.unwrap();
	assert_eq!(
		PublicKey::from_slice(&addrs[0].pub_key).unwrap(),
		public_key
	);

	let auth_tok_fail = AuthToken::new(secret_key, "origin.com", "create_key", 15, 16);

	let res_fail = TestClient::post(format!(
		"http://{}/key",
		&std::env::var("BIND_ADDR").unwrap()
	))
	.json(&auth_tok_fail)
	.send(comn_broker::route())
	.await;

	assert_eq!(res_fail.status_code.unwrap(), StatusCode::UNAUTHORIZED);

	Ok(())
}


#[sqlx::test(migrator = "comn_broker::MIGRATOR")]
async fn test_register_key_with_body(_pool: PgPool) -> sqlx::Result<()> {
	common::setup(_pool).await;
	let register_key_req = RegisterKeyReq {
		name: Some("Onion".to_string()),
	};
	let mut res = TestClient::post(format!(
		"http://{}/key",
		&std::env::var("BIND_ADDR").unwrap()
	))
	.add_header(
		AUTHORIZATION,
		&make_auth_header("NewKey", "comn.opus.ai", "create_key", 15, 5),
		true,
	)
	.json(&register_key_req)
	.send(comn_broker::route())
	.await;
	let body = res.take_json::<RegResp>().await.unwrap();

	let res2 = TestClient::get(format!(
		"http://{}/addr?addr={}",
		&std::env::var("BIND_ADDR").unwrap(),
		body.addr
	))
	.send(comn_broker::route())
	.await;
	// println!("res2 {:?}", res2.body.name);
	assert_eq!(res2.status_code.unwrap(), StatusCode::OK);


	//TO-DO check if name in response is equals to Onion
	// #[derive(Serialize, Deserialize, Debug)]
	// pub struct AddrRes {
	// 	pub id: String,
	// 	pub name: String,
	// 	pub created: String,
	// }
	// // tried using Addr but was getting the error expected UUID string
	// let name = res2.take_json::<Addr>().await.unwrap();
	// println!("name {:?}", name);
	// assert_eq!(register_key_req.name.unwrap().to_string(), name.to_string());

	Ok(())
}

#[sqlx::test(fixtures("addr_key"), migrator = "comn_broker::MIGRATOR")]
async fn test_get_addr(_pool: PgPool) -> sqlx::Result<()> {
	common::setup(_pool).await;

	let addr_req = TestClient::get(format!(
		"http://{}/addr?addr=≈a",
		&std::env::var("BIND_ADDR").unwrap()
	))
	.send(comn_broker::route())
	.await
	.take_json::<Vec<AddrRes>>()
	.await
	.unwrap();
	println!("addr_req {:?}", addr_req);
	assert_eq!(
		addr_req[0].addr.to_string(),
		ComnAddr::new("≈A").unwrap().to_string()
	);
	assert_eq!(
		addr_req[0].name.clone().unwrap(),
		"onion"
	);

	let name_req = TestClient::get(format!(
		"http://{}/addr?name=onion",
		&std::env::var("BIND_ADDR").unwrap()
	))
	.send(comn_broker::route())
	.await
	.take_json::<Vec<AddrRes>>()
	.await
	.unwrap();
	assert_eq!(
		name_req[0].addr.to_string(),
		ComnAddr::new("≈A").unwrap().to_string()
	);

	let addr_name_req = TestClient::get(format!(
		"http://{}/addr?name=onion&addr=≈a",
		&std::env::var("BIND_ADDR").unwrap()
	))
	.send(comn_broker::route())
	.await
	.take_json::<Vec<AddrRes>>()
	.await
	.unwrap();
	assert_eq!(
		addr_name_req[0].addr.to_string(),
		ComnAddr::new("≈A").unwrap().to_string()
	);

	let header_req = TestClient::get(format!(
		"http://{}/addr",
		&std::env::var("BIND_ADDR").unwrap()
	))
	.add_header(
		AUTHORIZATION,
		&make_auth_header(
			"NewKey",
			"comn.opus.ai",
			"get_addr",
			60 * 60 * 24 * 30,
			0,
		),
		true,
	)
	.send(comn_broker::route())
	.await
	.take_json::<Vec<AddrRes>>()
	.await
	.unwrap();
	assert_eq!(
		header_req[0].addr.to_string(),
		ComnAddr::new("≈A").unwrap().to_string()
	);

	let header_req = TestClient::get(format!(
		"http://{}/addr?name=onion",
		&std::env::var("BIND_ADDR").unwrap()
	))
	.add_header(
		AUTHORIZATION,
		&make_auth_header(
			"NewKey",
			"comn.opus.ai",
			"get_addr",
			60 * 60 * 24 * 30,
			0,
		),
		true,
	)
	.send(comn_broker::route())
	.await
	.take_json::<Vec<AddrRes>>()
	.await
	.unwrap();
	assert_eq!(
		header_req[0].addr.to_string(),
		ComnAddr::new("≈A").unwrap().to_string()
	);
	
	Ok(())
}


#[sqlx::test(fixtures("addr_key"), migrator = "comn_broker::MIGRATOR")]
async fn test_get_key(_pool: PgPool) -> sqlx::Result<()> {
	common::setup(_pool).await;

	let (_secret_key, public_key) = get_keys("NewKey");
	let addrs = TestClient::get(format!(
		"http://{}/key?addr=≈a",
		&std::env::var("BIND_ADDR").unwrap()
	))
	.send(comn_broker::route())
	.await
	.take_json::<Vec<AddrKey>>()
	.await
	.unwrap();
	assert_eq!(addrs[0].key_type, KeyType::Operation);
	assert_eq!(
		PublicKey::from_slice(&addrs[0].pub_key).unwrap(),
		public_key
	);
	Ok(())
}