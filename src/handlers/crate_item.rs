use salvo::http::{StatusCode};
use salvo::prelude::{handler, Depot, Response, Request};
use secp256k1::PublicKey;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use sqlx::types::{Uuid};
use sqlx::FromRow;
use crate::{
	AddCrateItemReq, write_file, CrateItemStorage,
	CrateItem, CrateItemRes, AccessType, SpecialAddr,
	read_file, verify_hash,
	comn_addr::ComnAddr, db::{db, get_config},
	read::{
		crate_item::CrateItemFilter,
		crates::CrateFilter,
		addr::AddrFilter
	},
	add::{
		crate_item::{AddCrateItem, AddCrateItemErr},
	},
};
use std::convert::Infallible;
use std::time::Duration;
use futures_util::StreamExt;
use salvo::sse::{self, SseEvent};
use tokio::time::interval;
use tokio_stream::wrappers::IntervalStream;
use crate::print_current_db;

#[handler]
pub async fn list_crate(req: &mut Request, res: &mut Response, depot: &mut Depot) {
	// TO-DO order by date and pagination, return unauthorize if don't have access but not when crate is empty
	print_current_db().await;
	if let Some(crate_id) = req.query::<&str>("id") {
		let pub_key = depot.get::<PublicKey>("public_key").unwrap();
		let mut filter = CrateItemFilter {
			addr: None,
			pub_key: Some(*pub_key),
			crate_id: crate_id.to_string(),
			per_page: 50,
			page_no: 0,
			result: None
		};
		if let Ok(_) = filter.init().await {
			let response = filter.to_res().unwrap();
			res.render(serde_json::to_string(&response).unwrap());
		} else {
			res.render(StatusCode::UNAUTHORIZED);
		}
	} else {
		res.render(StatusCode::BAD_REQUEST);
	}
}

#[handler]
pub async fn list_crate_stream(req: &mut Request, res: &mut Response, depot: &mut Depot) {
	let crate_id = req.query::<&str>("id");
	let pub_key = depot.get::<PublicKey>("public_key").unwrap();
	let mut filter = CrateItemFilter {
        addr: None,
        pub_key: Some(*pub_key),
        crate_id: crate_id.unwrap().to_string(),
        per_page: 50,
        page_no: 0,
        result: None
    };
    let _ = filter.init().await;
    let event_stream = {
        let mut response: Vec<CrateItemRes> = Vec::new();
        let interval = interval(Duration::from_secs(1));
        let stream = IntervalStream::new(interval);
        stream.map(move |_| {
            filter.update_res();
            if response == filter.to_res().unwrap() {
            	Ok::<SseEvent, Infallible>(SseEvent::default())
            } else {
            	response = filter.to_res().unwrap();
        		Ok(SseEvent::default().json(response.clone()).unwrap())
            }
        })
    };
    sse::stream(res, event_stream);
}



#[derive(FromRow, Serialize, Deserialize, Debug)]
pub struct CrateItemChunk {
	pub id: Uuid,
	pub sha2_hash: Vec<u8>,
	pub crate_item_id: Uuid,
	pub size_hectobyte: i16,
}

/// Gets crate item by key
///
/// Retruns http status code REQUEST_TIMEOUT if the signature is expired or is constructed in the future
/// Returns http status code NOT_ACCEPTABLE if the signature is invalid
/// On success, returns http status code OK and the data, created and update timestamp
#[handler]
pub async fn get_crate_item(req: &mut Request, res: &mut Response, depot: &mut Depot) {
	// print_current_db().await;
	if let Some(id) = req.query::<&str>("id") {
		// let auth_tok = depot.get::<AuthToken>("auth_token").unwrap();
		let qry = (|depot: &mut Depot, id| {
			use hex;
			let mut key: [u8; 33] = [0; 33];
			if *depot.get::<bool>("origin_matched").unwrap_or(&false) {
				if let Ok(pub_key) = depot.get::<PublicKey>("public_key") {
					key = pub_key.serialize();
				}
			}
			format!(
				"
SELECT ca.addr_id, ca.type, ci.*, it.media_type, NULL as data_file, s.scope_type as scope,
(SELECT COUNT(*) FROM crate_item_chunk WHERE crate_item_id = ci.id)::SMALLINT AS chunk_count
FROM crate_item ci
JOIN item_type it ON it.id = ci.type_id
JOIN scope s ON s.id = ci.scope_id
JOIN crate b ON b.id = ci.crate_id
JOIN crate_access ca ON ca.crate_id = b.id
LEFT JOIN addr_key ak on ak.addr_id = ca.addr_id
LEFT JOIN key k on k.id = ak.key_id
WHERE (k.pub_key = '\\x{}' or ca.addr_id = 'ffffffffffffffffffffffffffffffff')
and ca.type is not null and ci.id = '{}'::uuid;",
				hex::encode(key),
				id
			)
		})(depot, id);
		// println!("{}", qry);
		match sqlx::query_as::<_, CrateItem>(&qry)
			.fetch_one(db().await)
			.await
		{
			Ok(mut item) => {
				if item.chunk_count > 0 {
					let conf = get_config().await;
					if let Ok(data_file) =
						read_file(&format!("{}/{}_{}", conf.chunks_location, item.id, 0)).await
					{
						item.data_file = Some(data_file);
					}
				}
				res.render(serde_json::to_string(&item).unwrap())
			}
			Err(_e) => {
				println!("{} {:?}", qry, _e);
				res.render(StatusCode::NOT_FOUND)
			}
		}
	} else {
		res.render(StatusCode::BAD_REQUEST);
	}
}

#[handler]
pub async fn add_crate_item(req: &mut Request, res: &mut Response, depot: &mut Depot) {
	// print_current_db().await;
	let scope = depot.get::<Vec<String>>("scope").unwrap();
	if scope[0]!="crate_write" {
		res.render(StatusCode::BAD_REQUEST);
		return;
	}
	if let Ok(crate_req) = req.parse_json::<AddCrateItemReq>().await {
		let pub_key = depot.get::<PublicKey>("public_key").unwrap();
		let mut verify_addr = AddrFilter {
			name: None,
			addr: Some(crate_req.addr.clone()),
			keys: Some(vec!(*pub_key)),
			result: None,
		};
		if let Ok(_) = verify_addr.init().await {
			let mut crate_access = CrateFilter {
				name: None,
				addr: Some(vec!(
					SpecialAddr::Registered.value(),
					SpecialAddr::Public.value(),
					crate_req.addr.clone(),
				)),
				pub_key: None,
				crate_id: Some(crate_req.crate_id.clone()),
				access_type: vec!(
					AccessType::Owner,
				    AccessType::Admin,
				    AccessType::Editor,
				    AccessType::Writer
				),
				result: None,
			};
			if let Ok(_) = crate_access.init().await {
				let add_item = AddCrateItem {
					crate_id: Uuid::parse_str(&crate_req.crate_id).unwrap(),
					addr: crate_req.addr,
					item_path: crate_req.item_path,
					media_type: crate_req.media_type,
					sha2_hash: crate_req.sha2_hash,
					data: crate_req.data,
					scope: scope[1].to_string(),
				};
				match add_item.add().await {
					Ok(rr) => res.render(serde_json::to_string(&rr).unwrap()),
					Err(AddCrateItemErr::PayloadLarge) => res.render(StatusCode::PAYLOAD_TOO_LARGE),
					Err(AddCrateItemErr::InternalErr) => res.render(StatusCode::INTERNAL_SERVER_ERROR),
				}
			} else {
				res.render(StatusCode::BAD_REQUEST);
			}
		} else {
			res.render(StatusCode::BAD_REQUEST);
		}	
	} else {
		res.render(StatusCode::BAD_REQUEST);
	}
}

// pub struct AddFileChunkReq {
// 	pub id: i16,
// 	pub crate_item_id: Uuid,
// 	pub sha2_hash: Vec<u8>,
// 	pub data: Vec<u8>,
// }

// #[handler]
// pub async fn add_crate_item_chunk(req: &mut Request, res: &mut Response, depot: &mut Depot) {
// 	let fdata = crate_req.data;
// 	if let Ok(fname) = write_file(&fdata, &crate_req.media_type).await {
// 	}
// 	let conf = get_config().await;
// }