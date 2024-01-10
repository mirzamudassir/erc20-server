use chrono::{DateTime, Utc};
use salvo::http::{StatusCode};
use salvo::prelude::{handler, Depot, Response, Request};
use secp256k1::PublicKey;
use serde::{Deserialize, Serialize};
use sqlx::types::{Uuid};
use sqlx::FromRow;
use crate::{
	AccessType, SpecialAddr,
	AddCrateReq, _add_crate, CrateAccess,
	comn_addr::ComnAddr, db::{db},
	read::crates::{CrateOwnerFilter, CrateFilter}
};
use crate::print_current_db;


#[handler]
pub async fn add_crate(req: &mut Request, res: &mut Response, depot: &mut Depot) {
	// print_current_db().await;
	if let Ok(crate_req) = req.parse_json::<AddCrateReq>().await {
		let pub_key = depot.get::<PublicKey>("public_key").unwrap();
		let mut tx = db().await.begin().await.unwrap();
		if let Ok(_addr) = sqlx::query_scalar::<_, Uuid>(
			"
			SELECT a.id from addr a JOIN addr_key ak ON ak.addr_id  = a.id
			JOIN key k ON ak.key_id  = k.id WHERE k.pub_key = $1 AND a.id = $2::uuid",
		)
		.bind(pub_key.serialize())
		.bind(crate_req.addr.to_uuid())
		.fetch_one(&mut *tx)
		.await {
			let rr = _add_crate(crate_req).await;
			res.render(serde_json::to_string(&rr).unwrap());
		} else {
			res.render(StatusCode::BAD_REQUEST);
		}
	} else {
		res.render(StatusCode::BAD_REQUEST);
	}
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GetCrateReq {
	pub name: Option<String>,
	pub addr: Option<ComnAddr>,
	pub access: Option<Vec<AccessType>>,
	pub id: Option<String>,
}

#[handler]
pub async fn get_crate(req: &mut Request, res: &mut Response, depot: &mut Depot) {
	print_current_db().await;
	// if id is present in the req. we are only using it to check if crate 
	// exists and user has access to it.
	if let Ok(crate_req) = req.parse_queries::<GetCrateReq>() {
		let crate_id: String;
		println!("crate_id {:?}", crate_req);
		if crate_req.id == None {
			// getting the crate id if none is provided
			let mut crate_filter = CrateFilter {
				name: crate_req.name,
				addr: Some(vec!(crate_req.addr.unwrap())),
				pub_key: None,
				crate_id: None,
				access_type: crate_req.access.unwrap(),
				result: None,
			};
			if let Ok(_) = crate_filter.init().await {
				if let Ok(crates) = crate_filter.get_crate() {
					crate_id = crates[0].id.to_string();
				} else {
					res.render(StatusCode::BAD_REQUEST);
					return;
				}
			} else {
				res.render(StatusCode::NOT_FOUND);
				return;
			}
		} else {
			crate_id = crate_req.id.unwrap();
		}
		let pub_key = depot.get::<PublicKey>("public_key").unwrap();
		let mut filter = CrateFilter {
			name: None,
			addr: Some(vec!(SpecialAddr::Registered.value())),
			pub_key: Some(*pub_key),
			crate_id: Some(crate_id),
			access_type: vec!(
				AccessType::Owner,
			    AccessType::Admin,
			    AccessType::Editor,
			    AccessType::Reader,
			    AccessType::Writer
			),
			result: None,
		};
		if let Ok(_) = filter.init().await {
			let rr = filter.get_crate().unwrap();
			res.render(serde_json::to_string(&rr[0]).unwrap());
		} else {
			res.render(StatusCode::BAD_REQUEST);
		}
	} else {
		res.render(StatusCode::BAD_REQUEST);
	}
}



#[handler]
pub async fn list_crates(res: &mut Response, depot: &mut Depot) {
	// only listing crates that have explicit access. public & registered crates are not listed
	// print_current_db().await;
	let pub_key = depot.get::<PublicKey>("public_key").unwrap();
	let mut filter = CrateFilter {
		name: None,
		addr: None,
		pub_key: Some(*pub_key),
		crate_id: None,
		access_type: vec!(
			AccessType::Owner,
		    AccessType::Admin,
		    AccessType::Editor,
		    AccessType::Reader,
		    AccessType::Writer
		),
		result: None,
	};
	let _ = filter.init().await;
	let rr = filter.result.unwrap();

	res.render(serde_json::to_string(&rr).unwrap());
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CrateAccessReq {
	pub crate_id: String,
	pub access_to_addr: String,
	pub give_access: bool,
	pub access_type: AccessType,
	pub expires: Option<DateTime<Utc>>,
}



#[derive(FromRow, Serialize, Deserialize, Debug)]
pub struct CrateAdmin {
	pub addr: Uuid,
	pub r#type: AccessType,
	pub pub_key: Option<Vec<u8>>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CrateAdmins {
	pub list: Vec<CrateAdmin>,
}

impl CrateAdmins {
	pub fn match_pub_key(&self, pub_key_req: Vec<u8>) -> bool {
		for i in self.list.iter() {
	      if i.pub_key == Some(pub_key_req.clone()) { return true; }
	    }
    false
	}

	pub fn owner_count(&self) -> u8 {
		let mut count = 0;
		// println!("list {:?}", self.list);
		for i in self.list.iter() {
	      if i.r#type == AccessType::Owner { count+=1; }
	    }
		// println!("count {}", count);
    	
    	count
	}

}

#[handler]
pub async fn change_crate_access(req: &mut Request, res: &mut Response, depot: &mut Depot) {
	// print_current_db().await;
	let mut tx = db().await.begin().await.unwrap();
	let pub_key = depot.get::<PublicKey>("public_key").unwrap();

	if let Ok(mut crate_req) = req.parse_json::<CrateAccessReq>().await {
		let ownership = CrateAdmins {
			list: sqlx::query_as::<_, CrateAdmin>(
					"
					SELECT a.id as addr, ca.type, k.pub_key
					FROM addr a 
					LEFT JOIN addr_key ak ON ak.addr_id = a.id
					LEFT JOIN key k on k.id = ak.key_id
					JOIN crate_access ca ON ca.addr_id = a.id
					JOIN crate c ON c.id = ca.crate_id
					WHERE ca.type in ('owner', 'admin')
					AND c.id = $1::uuid
					"
				)
				.bind(crate_req.crate_id.clone())
				.fetch_all(&mut *tx)
				.await
				.unwrap(),
		};

		if ownership.match_pub_key(pub_key.serialize().to_vec()) {
			let comn_addr = ComnAddr::new(&crate_req.access_to_addr).unwrap().to_uuid();
			let entry: CrateAccess;

			if crate_req.give_access {
				entry = sqlx::query_as::<_, CrateAccess>(
					"
					INSERT INTO crate_access(crate_id, addr_id, type, expires)
					VALUES($1::uuid, $2::uuid, $3, $4) RETURNING *
					"
				)
				.bind(crate_req.crate_id)
				.bind(comn_addr)
				.bind(crate_req.access_type)
				.bind(crate_req.expires)
				.fetch_one(&mut *tx)
				.await
				.unwrap();
			} else {
				if crate_req.access_type == AccessType::Owner && ownership.owner_count()==1{
					res.render(StatusCode::BAD_REQUEST);
					return;
				} else {
					entry = sqlx::query_as::<_, CrateAccess>(
						"
						DELETE FROM crate_access
						WHERE crate_id = $1::uuid AND addr_id = $2::uuid
						AND type = $3
						RETURNING *
						"
					)
					.bind(crate_req.crate_id)
					.bind(comn_addr)
					.bind(crate_req.access_type)
					.fetch_one(&mut *tx)
					.await
					.unwrap();
				}
			}
			// println!("entry {:?}", entry);
			tx.commit().await.unwrap();
			res.render(serde_json::to_string(&entry).unwrap());
		} else {
			res.render(StatusCode::UNAUTHORIZED);
		}
	} else {
		res.render(StatusCode::BAD_REQUEST);
	}
}