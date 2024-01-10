use salvo::http::{StatusCode};
use salvo::prelude::{handler, Depot, Response};
use secp256k1::PublicKey;
use crate::{
	db::{db},
	read::addr::AddrFilter,
	update::coin::{Transaction, TransactionErr},
};
use crate::print_current_db;

#[handler]
pub async fn transaction(res: &mut Response, depot: &mut Depot) {
	print_current_db().await;
	let pub_key = depot.get::<PublicKey>("public_key").unwrap();
	let req_json = depot.get::<String>("req").unwrap();

	if let Ok(transaction_req) = serde_json::from_str::<Transaction>(req_json) {
		let mut verify_sender = AddrFilter {
			name: None,
			addr: Some(transaction_req.sender.clone()),
			keys: Some(vec!(*pub_key)),
			result: None,
		};
		if let Ok(_) = verify_sender.init().await {
			match transaction_req.transfer_coins().await {
				Ok(result) => res.render(serde_json::to_string(&result).unwrap()), 
				Err(TransactionErr::AlreadyReported) => res.render(StatusCode::ALREADY_REPORTED),
				Err(TransactionErr::LowAmount) | Err(TransactionErr::BadData) =>
					res.render(StatusCode::BAD_REQUEST),
			}
		} else {
			res.render(StatusCode::BAD_REQUEST);
		}
	} else {
		res.render(StatusCode::BAD_REQUEST);
	}
}

#[handler]
pub async fn get_comn_coins(res: &mut Response, depot: &mut Depot) {
	let mut tx = db().await.begin().await.unwrap();
	let pub_key = depot.get::<PublicKey>("public_key").unwrap();
	let result = sqlx::query_scalar::<_, i64>(
		"
		SELECT (WITH account AS (
					SELECT (
						SELECT a.id
						from addr a
						JOIN addr_key ak ON ak.addr_id = a.id
						JOIN key k ON k.id = ak.key_id 
						WHERE k.pub_key = $1
					)::text
				)
				SELECT 
					ci.data_json ->> (SELECT * from account)
				from crate_item ci
				WHERE id = '000000000000000000000000000001cc'::uuid)::int8
		"
	)
	.bind(pub_key.serialize())
	.fetch_one(&mut *tx)
	.await;
	let amount = match result {
		Ok(r) => r,
		Err(..) => 0,
	};
	res.render(serde_json::to_string(&amount).unwrap());
}