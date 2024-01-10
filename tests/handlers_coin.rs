mod common;
use common::{get_keys, make_auth_header};
use comn_broker::{
	auth_token::{ProtectedReq, Protected}, comn_addr::ComnAddr,
};
use comn_broker::update::{
	coin::{Transaction}
};
use salvo::http::header::AUTHORIZATION;
use salvo::prelude::*;
use salvo::test::{ResponseExt, TestClient};
use sqlx::PgPool;
use serde::{Deserialize, Serialize};


#[sqlx::test(fixtures("comn_coin"), migrator = "comn_broker::MIGRATOR")]
async fn test_get_comn_coins(_pool: PgPool) -> sqlx::Result<()> {
	common::setup(_pool).await;

	let req = TestClient::get(format!(
		"http://{}/comn",
		&std::env::var("BIND_ADDR").unwrap()
	))
	.add_header(
		AUTHORIZATION,
		&make_auth_header(
			"NewKey",
			"comn.opus.ai",
			"account_amount",
			60 * 60 * 24 * 30,
			0,
		),
		true,
	)
	.send(comn_broker::route())
	.await
	.take_json::<i64>()
	.await
	.unwrap();
	assert_eq!(req,100000000000);

	let req2 = TestClient::get(format!(
		"http://{}/comn",
		&std::env::var("BIND_ADDR").unwrap()
	))
	.add_header(
		AUTHORIZATION,
		&make_auth_header(
			"Key1",
			"comn.opus.ai",
			"account_amount",
			60 * 60 * 24 * 30,
			0,
		),
		true,
	)
	.send(comn_broker::route())
	.await
	.take_json::<i64>()
	.await
	.unwrap();

	assert_eq!(req2,0);
	
	Ok(())
}

#[derive(Serialize, Deserialize, Debug)]
pub struct BadTransactionReq {
	pub amount: i64,
	pub receiver: ComnAddr,
	pub sender: ComnAddr,
	pub comment: Option<String>,
	pub nonce: String,
}

#[sqlx::test(fixtures("transaction"), migrator = "comn_broker::MIGRATOR")]
async fn test_transaction(_pool: PgPool) -> sqlx::Result<()> {
	common::setup(_pool).await;
	let mut transaction = Transaction {
		amount: 100,
		receiver: ComnAddr::new("≈a").unwrap(),
		sender: ComnAddr::new("≈6D").unwrap(),
		comment: None,
		nonce: "asdf2feNSok98Ingp".to_string()
	};
	let (secret_key, _public_key) = get_keys("NewKey");

	let t = serde_json::to_string(&transaction).unwrap();
	let protected = Protected::new(t, secret_key);
	let req = ProtectedReq::from(protected);
	// println!("req {:?}", req);
	let res = TestClient::post(format!(
		"http://{}/comn/transaction",
		&std::env::var("BIND_ADDR").unwrap()
	))
	.add_header(
		AUTHORIZATION,
		&make_auth_header(
			"NewKey",
			"comn.opus.ai",
			"transaction",
			60 * 60 * 24 * 30,
			0,
		),
		true,
	)
	.json(&req)
	.send(comn_broker::route())
	.await;
	// println!("res {:?}", res);
	assert_eq!(res.status_code.unwrap(), StatusCode::OK);

	// using some nonce
	transaction.amount = 10;
	transaction.comment = Some("req with some nonce".to_string());
	let t = serde_json::to_string(&transaction).unwrap();
	let protected = Protected::new(t, secret_key);
	let req = ProtectedReq::from(protected);
	let res = TestClient::post(format!(
		"http://{}/comn/transaction",
		&std::env::var("BIND_ADDR").unwrap()
	))
	.add_header(
		AUTHORIZATION,
		&make_auth_header(
			"NewKey",
			"comn.opus.ai",
			"transaction",
			60 * 60 * 24 * 30,
			0,
		),
		true,
	)
	.json(&req)
	.send(comn_broker::route())
	.await;
	// println!("res {:?}", res);
	assert_eq!(res.status_code.unwrap(), StatusCode::ALREADY_REPORTED);


	// amount more than balance
	transaction.amount = 100000000001;
	transaction.nonce = "asdf2feNSok98Indp".to_string();
	let t = serde_json::to_string(&transaction).unwrap();
	let protected = Protected::new(t, secret_key);
	let req = ProtectedReq::from(protected);
	let res_fail = TestClient::post(format!(
		"http://{}/comn/transaction",
		&std::env::var("BIND_ADDR").unwrap()
	))
	.add_header(
		AUTHORIZATION,
		&make_auth_header(
			"NewKey",
			"comn.opus.ai",
			"transaction",
			60 * 60 * 24 * 30,
			0,
		),
		true,
	)
	.json(&req)
	.send(comn_broker::route())
	.await;
	// println!("res {:?}", res_fail);
	assert_eq!(res_fail.status_code.unwrap(), StatusCode::BAD_REQUEST);

	//amount negitive
	let bad_transaction = BadTransactionReq {
		amount: -100,
		receiver: ComnAddr::new("≈a").unwrap(),
		sender: ComnAddr::new("≈6D").unwrap(),
		comment: None,
		nonce: "asdf2feNasd098Ingp".to_string()
	};
	let t = serde_json::to_string(&bad_transaction).unwrap();
	let protected = Protected::new(t, secret_key);
	let req = ProtectedReq::from(protected);
	let bad_res = TestClient::post(format!(
		"http://{}/comn/transaction",
		&std::env::var("BIND_ADDR").unwrap()
	))
	.add_header(
		AUTHORIZATION,
		&make_auth_header(
			"NewKey",
			"comn.opus.ai",
			"transaction",
			60 * 60 * 24 * 30,
			0,
		),
		true,
	)
	.json(&req)
	.send(comn_broker::route())
	.await;
	// println!("res {:?}", bad_res);
	assert_eq!(bad_res.status_code.unwrap(), StatusCode::BAD_REQUEST);
	
	Ok(())
}