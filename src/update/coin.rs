use chrono::{DateTime, Utc};
use std::{error::Error, fmt};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use sqlx::types::{Uuid, Json};
use sqlx::FromRow;
use crate::{
	AccessType, AddCrateItemReq, write_file, CrateItemStorage,
	AddCrateReq, _add_crate, CrateAccess, SpecialAddr,
	comn_addr::ComnAddr, db::{db, get_config},
	read::crates::{CrateOwnerFilter, CrateFilterErr, CrateFilter},
	add::crate_item::AddCrateItem,
};
// use crate::print_current_db;

#[derive(Serialize, Deserialize, Debug)]
pub struct Transaction {
	pub amount: u64,
	pub receiver: ComnAddr,
	pub sender: ComnAddr,
	pub comment: Option<String>,
	pub nonce: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TransactionHistory {
	pub amount: u64,
	pub date: DateTime<Utc>,
	pub credit: bool,
	pub comment: Option<String>,
	pub addr: ComnAddr
}

#[derive(Serialize, Deserialize, Debug, FromRow, PartialEq, PartialOrd)]
pub struct TransactionQuery {
	pub amount: i64,
	pub sender: String,
	pub receiver: String,
}

#[derive(Debug)]
pub enum TransactionErr {
	AlreadyReported,
	BadData,
	LowAmount,
}

impl Error for TransactionErr {}

impl fmt::Display for TransactionErr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            TransactionErr::BadData => write!(f, "addrs are not right."),
            TransactionErr::LowAmount => write!(f, "Amount to transfer is more than balance."),
            TransactionErr::AlreadyReported => write!(f, "A transaction has already happended with the nonce.")
        }
    }
}

impl Transaction {
	pub async fn transfer_coins(&self) -> Result<TransactionQuery, TransactionErr> {
		let mut tx = db().await.begin().await.unwrap();
		if let Ok(_nonce) = sqlx::query_scalar::<_, String>(
			"
			SELECT ci.data_json ->> $1::text
			FROM crate_item ci
			WHERE 
				ci.id = '000000000000000000000000000011cc'::uuid
			"
		)
		.bind(self.nonce.clone())
		.fetch_one(&mut *tx)
		.await {
			return Err(TransactionErr::AlreadyReported);
		} else {
			let result = sqlx::query_as::<_, TransactionQuery>(
				"	
				WITH sender AS (
					SELECT $1::text
				), receiver AS (
					SELECT (
						SELECT a.id
						from addr a
						WHERE a.id = $2::uuid
					)::text
				), new_sender_amount AS (
					SELECT ((
						SELECT (
							SELECT ci.data_json
							from crate_item ci
							WHERE ci.id = '000000000000000000000000000001cc'::uuid
						)::jsonb ->> (SELECT * from sender)
					)::int8) - $3::int8
				)
				UPDATE crate_item
				SET
					data_json =
					CASE
						WHEN (select * from new_sender_amount)>=0
						THEN
							jsonb_set(
								jsonb_set(
									data_json,
									ARRAY[(SELECT * from receiver)], 
									(((COALESCE((SELECT ((
										data_json
									)::jsonb ->> (SELECT * from receiver)))::numeric,0) + $3::numeric)::text)::jsonb)
								),
								ARRAY[(SELECT * from sender)], 
								((SELECT * from new_sender_amount)::text)::jsonb
							)
						ELSE
							data_json
					END
				WHERE 
					id = '000000000000000000000000000001cc'::uuid
				RETURNING (SELECT * from new_sender_amount) as amount, (SELECT * from sender) as sender,
				 (SELECT * from receiver) as receiver
				"
			)
			.bind(self.sender.to_uuid())
			.bind(self.receiver.to_uuid())
			.bind(self.amount as i64)
			.fetch_one(&mut *tx)
			.await
			.unwrap();
			tx.commit().await.unwrap();
			println!("result {:?}", result);
			if result.amount<0 {
				return Err(TransactionErr::LowAmount);
			} else {
				let transaction_time = chrono::offset::Utc::now();
				let mut tx = db().await.begin().await.unwrap();		
				let _updated_json = sqlx::query_scalar::<_, Uuid>(
					"
					UPDATE crate_item
					SET
						data_json =
							jsonb_set(
								data_json,
								$1::text[], 
								$2
							)
					WHERE 
						id = '000000000000000000000000000011cc'::uuid
					RETURNING
						id
					"
				)
				.bind([self.nonce.to_string()])
				.bind(Json::from(transaction_time.to_string()))
				.fetch_one(&mut *tx)
				.await
				.unwrap();
				let sender_history = TransactionHistory {
					amount: self.amount,
					date: transaction_time,
					credit: false,
					comment: self.comment.clone(),
					addr: self.receiver.clone()
				};
				let mut sender_crate = CrateFilter {
					name: Some(ComnAddr::from_uuid(&result.sender).unwrap().to_string()),
					addr: Some(vec!(SpecialAddr::ComnCoin.value())),
					pub_key: None,
					crate_id: None,
					access_type: vec!(AccessType::Owner),
					result: None,
				};
				let _ = sender_crate.init().await;
				let sender_crate_id = sender_crate.get_crate().unwrap()[0].id;
				let _rr = add_transaction_json(sender_history, sender_crate_id).await;

				let receiver_history = TransactionHistory {
					amount: self.amount,
					date: transaction_time,
					credit: true,
					comment: self.comment.clone(),
					addr: ComnAddr::from_uuid(&result.sender.to_string()).unwrap()
				};
				let mut receiver_crate = CrateOwnerFilter {
					addr: Some(ComnAddr::from_uuid("000000000000000000000000000000cc").unwrap().to_string()),
					name: Some(ComnAddr::from_uuid(&result.receiver.to_string()).unwrap().to_string()),
					crate_ids: None,
				};
				let receiver_crate_id = match receiver_crate.get_id().await {
					Ok(result) => result,
					Err(CrateFilterErr::NotFound) => {
						let create_crate_req = AddCrateReq {
							name: receiver_crate.name.unwrap(),
							comment: "ComnCoin transaction history".to_string(),
							addr: ComnAddr::new(&receiver_crate.addr.unwrap()).unwrap(),
							expires: None
						};
						let new_crate = _add_crate(create_crate_req).await;
						sqlx::query_as::<_, CrateAccess>(
							"
							INSERT INTO crate_access(crate_id, addr_id, type)
							VALUES($1::uuid, $2::uuid, $3) RETURNING *
							"
						)
						.bind(new_crate.id)
						.bind(result.receiver.to_string())
						.bind(AccessType::Reader)
						.fetch_one(&mut *tx)
						.await
						.unwrap();
						new_crate.id
					},
					Err(CrateFilterErr::BadData) | Err(CrateFilterErr::Multiple) 
						=> panic!("While retriving receiver crate."),
				};
				tx.commit().await.unwrap();
				let receiver_crate_item = add_transaction_json(receiver_history, receiver_crate_id).await;
				println!("rr {:?}", receiver_crate_item);

				Ok(result)
			}
		}
	}
}

async fn add_transaction_json(history: TransactionHistory, crate_id: Uuid) -> Uuid {
	let item_path = "/".to_string() + &history.date.to_string();
	let history_json = serde_json::to_string(&history).unwrap();
	let history_data = history_json.as_bytes();
	let mut hasher = Sha256::new();
	hasher.update(history_data);
	let sha2_hash = hasher.finalize().as_slice().to_vec();

	let add_item = AddCrateItem {
		crate_id: crate_id,
		addr: SpecialAddr::ComnCoin.value(),
		item_path: item_path.to_string(),
		media_type: "application/json".to_string(),
		data: Some(history_data.into()),
		sha2_hash: sha2_hash,
		scope: "transaction_history".to_string(),
	};
	let rr = add_item.add().await.unwrap();

	rr
}