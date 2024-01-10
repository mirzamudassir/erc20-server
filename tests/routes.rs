// #[sqlx::test(fixtures("notification"), migrator = "comn_broker::MIGRATOR")]
// async fn test_notification(_pool: PgPool) -> sqlx::Result<()> {
// 	common::setup(_pool).await;

// 	let mut user_res = TestClient::post(format!(
// 		"http://{}/key",
// 		&std::env::var("BIND_ADDR").unwrap()
// 	))
// 	.add_header(
// 		AUTHORIZATION,
// 		&make_auth_header("NewKey", "comn.opus.ai", "create_key", 15, 5),
// 		true,
// 	)
// 	.send(comn_broker::route())
// 	.await;
// 	let user_body = user_res.take_json::<comn_broker::RegResp>().await.unwrap();
// 	// println!("body {:?}", user_body);
// 	let register_key_req = RegisterKeyReq {
// 		name: Some("ComnApp".to_string()),
// 	};
// 	let mut app_res = TestClient::post(format!(
// 		"http://{}/key",
// 		&std::env::var("BIND_ADDR").unwrap()
// 	))
// 	.add_header(
// 		AUTHORIZATION,
// 		&make_auth_header("NewKeyApp", "comn.opus.ai", "create_key", 15, 5),
// 		true,
// 	)
// 	.json(&register_key_req)
// 	.send(comn_broker::route())
// 	.await;
// 	let app_body = app_res.take_json::<comn_broker::RegResp>().await.unwrap();
// 	println!("body {:?}", app_body);

// 	let crate_req = AddCrateReq {
// 		name: "__notification".to_string(),
// 		comment: "".to_string(),
// 		addr: user_body.addr,
// 		expires: None,
// 	};
// 	let mut user_crate_res = TestClient::post(format!(
// 		"http://{}/crate",
// 		&std::env::var("BIND_ADDR").unwrap()
// 	))
// 	.add_header(
// 		AUTHORIZATION,
// 		&make_auth_header(
// 			"NewKey",
// 			"comn.opus.ai",
// 			"crate_write",
// 			60 * 60 * 24 * 30,
// 			0,
// 		),
// 		true,
// 	)
// 	.json(&crate_req)
// 	.send(comn_broker::route())
// 	.await;
// 	let crate_body = user_crate_res.take_json::<Crate>().await.unwrap();
// 	// println!("user crate {:?}", crate_body);

// 	let access_req = CrateAccessReq {
// 		crate_filter: CrateOwnerFilter {
// 			crate_ids: Some(vec![crate_body.id]),
// 			name: None,
// 			addr: None,
// 		},
// 		access_to_addr: "≈7ZZZZZZZZZZZZZZZZZZZZZZZZY".to_string(),
// 		give_access: true,
// 		access_type: AccessType::writer,
// 		expires: None,
// 	};
// 	let mut crate_access_res = TestClient::post(format!(
// 		"http://{}/crate/access",
// 		&std::env::var("BIND_ADDR").unwrap()
// 	))
// 	.add_header(
// 		AUTHORIZATION,
// 		&make_auth_header(
// 			"NewKey",
// 			"comn.opus.ai",
// 			"change_crate_access",
// 			60 * 60 * 24 * 30,
// 			0,
// 		),
// 		true,
// 	)
// 	.json(&access_req)
// 	.send(comn_broker::route())
// 	.await;
// 	println!("crate access res {:?}", crate_access_res);


// 	let data_json = "trust me bro".to_string();
// 	let data_bytes = data_json.as_bytes();
// 	let mut hasher = Sha256::new();
// 	println!("data_bytes {:?}", data_bytes);
// 	hasher.update(data_bytes);
// 	let sha2_hash = hasher.finalize().as_slice().to_vec();
// 	let add_item_req = AddCrateItemReq {
// 		crate_filter: CrateOwnerFilter {
// 			name: None,
// 			addr: None,
// 			crate_ids: Some(vec![crate_body.id]),
// 		},
// 		item_path: "/text".to_string(),
// 		media_type: "text/plain".to_string(),
// 		data: Some(data_bytes.into()),
// 		sha2_hash: sha2_hash,
// 	};

// 	let mut data_item_res = TestClient::post(format!(
// 		"http://{}/item",
// 		&std::env::var("BIND_ADDR").unwrap()
// 	))
// 	.add_header(
// 		AUTHORIZATION,
// 		&make_auth_header(
// 			"NewKeyApp",
// 			"comn.opus.ai",
// 			"crate_write",
// 			60 * 60 * 24 * 30,
// 			0,
// 		),
// 		true,
// 	)
// 	.json(&add_item_req)
// 	.send(comn_broker::route())
// 	.await;

// 	println!("data_item_resssssssssssssssssssssssss {:?}", data_item_res);

// 	let mut list_crate = TestClient::get(format!(
// 		"http://{}/crate/list?id={}",
// 		&std::env::var("BIND_ADDR").unwrap(),
// 		crate_body.id
// 	))
// 	.add_header(
// 		AUTHORIZATION,
// 		&make_auth_header("NewKey", "comn.opus.ai", "crate_read", 60 * 60 * 24 * 30, 0),
// 		true,
// 	)
// 	.send(comn_broker::route())
// 	.await;
// 	let crate_item_body = list_crate.take_json::<Vec<CrateItemRes>>().await.unwrap();
// 	println!("list crate {:?}", crate_item_body);
// 	assert_eq!(list_crate.status_code.unwrap(), StatusCode::OK);


// 	// let access_req2 = CrateAccessReq {
// 	// 	crate_id: crate_body.id,
// 	// 	access_to_addr: "≈7ZZZZZZZZZZZZZZZZZZZZZZZZY".to_string(),
// 	// 	give_access: false,
// 	// 	access_type: AccessType::writer,
// 	// 	expires: None,
// 	// };
// 	// let mut crate_access_res2 = TestClient::post(format!(
// 	// 	"http://{}/crate/access",
// 	// 	&std::env::var("BIND_ADDR").unwrap()
// 	// ))
// 	// .add_header(
// 	// 	AUTHORIZATION,
// 	// 	&make_auth_header(
// 	// 		"NewKey",
// 	// 		"comn.opus.ai",
// 	// 		"change_crate_access",
// 	// 		60 * 60 * 24 * 30,
// 	// 		0,
// 	// 	),
// 	// 	true,
// 	// )
// 	// .json(&access_req2)
// 	// .send(comn_broker::route())
// 	// .await;
// 	// println!("crate access res 2 {:?}", crate_access_res2);

// 	Ok(())
// }
