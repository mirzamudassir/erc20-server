use crate::db;
use chrono::{Duration, Utc};
use salvo::http::header;
use salvo::http::StatusCode;
use salvo::prelude::{handler, Depot, Request, Response};
use secp256k1::ecdsa::{RecoverableSignature, RecoveryId, Signature};
use secp256k1::{Message, PublicKey, SecretKey, SECP256K1};
use serde::{Deserialize, Serialize};
use sha3::{Digest, Keccak256};
use std::{str};
// use std::time::{Duration, SystemTime};

struct AuthData(AuthToken, PublicKey);
async fn update_auth(req: &mut Request, depot: &mut Depot) -> Result<AuthData, StatusCode> {
	// if let Some(Ok(host)) = req.headers().get(header::HOST).map(|host| host.to_str()) {
	// 	let config = db::get_config().await;
	// 	println!("update_auth Config={:?} {}", *config, config.host_names.contains(&host.to_string()));
	// }
	// for (k, hdr) in req.headers().iter() {
	// 	println!("update_auth {}={:?}", k, hdr);
	// }

	if let Some(Ok(auth)) = req
		.headers()
		.get(header::AUTHORIZATION)
		.map(|auth| auth.to_str())
	{
		if auth.starts_with("COMN") {
			let hv = auth.split_once(' ').map(|(_, token)| token.to_owned());
			let mut auth_tok = AuthToken::from(hv.unwrap()).unwrap();
			let pub_key = auth_tok.recover();

			let origin_match = db::get_config().await.host_names.contains(&auth_tok.origin);
			let scopes: Vec<String> = auth_tok.scope.split(",").map(|i| i.to_string()).collect();
			depot.insert("auth_token", auth_tok.clone());
			depot.insert("public_key", pub_key.unwrap());
			depot.insert("origin_matched", origin_match);
			depot.insert("scope", scopes);

			if origin_match && !auth_tok.expired() && pub_key.ok().is_some() {
				// println!("update_auth {:?} {:?} {}", auth_tok, pub_key, origin_match);
				return Ok(AuthData(auth_tok, pub_key.unwrap()));
			}
		}
	}
	Err(StatusCode::UNAUTHORIZED)
}

#[handler]
pub async fn check_auth(req: &mut Request, depot: &mut Depot) {
	let _ = update_auth(req, depot).await;
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ProtectedReq {
	pub msg: String, // json msg 
	pub proof: String
}

impl From<Protected> for ProtectedReq {
  fn from(r: Protected) -> Self {
  	let msg = r.req_json;
  	let proof = hex::encode(&r.proof);
	Self {
		msg: msg,
		proof: proof
	}
  }
}

#[derive(Debug)]
pub struct Protected {
	keccak: [u8; 32],
	proof: [u8; 64],
	pub req_json: String
}

impl Protected {
	pub fn new(json: String, secret_key: SecretKey) -> Self {
		let msg_bytes = json.as_bytes();
		let keccak: [u8; 32] = Keccak256::new_with_prefix(msg_bytes.clone())
			.finalize()
			.as_slice()
			.try_into()
			.expect("Wrong length");
		let msg = Message::from_slice(&keccak).unwrap();
		let sign = SECP256K1
		.sign_ecdsa(&msg, &secret_key)
		.serialize_compact();
		
		Self {
			keccak: keccak,
			proof: sign,
			req_json: json,
		}
	}

	pub fn verify(&self, pub_key: PublicKey) -> Result<(), secp256k1::Error> {
		let msg = Message::from_slice(&self.keccak).unwrap();
		let proof = Signature::from_compact(&self.proof).unwrap();
		SECP256K1.verify_ecdsa(
			&msg,
			&proof,
			&pub_key,
		)
	}
}

impl From<ProtectedReq> for Protected {
  fn from(r: ProtectedReq) -> Self {
  	let msg_bytes = r.msg.as_bytes();
  	let msg: [u8; 32] = Keccak256::new_with_prefix(msg_bytes.clone())
		.finalize()
		.as_slice()
		.try_into()
		.expect("Wrong length");
  	let sign = hex::decode(r.proof).unwrap();
  	let proof: [u8; 64] = sign.try_into().unwrap();
		Self {
			keccak: msg,
			proof: proof,
			req_json: r.msg,
		}
  }
}

#[handler]
pub async fn protected(req: &mut Request, depot: &mut Depot, res: &mut Response) {
	if let Ok(_auth_data) = update_auth(req, depot).await {
		let protected_req = req.parse_json::<ProtectedReq>().await.unwrap();
		let pub_key = depot.get::<PublicKey>("public_key").unwrap();
		let p = Protected::from(protected_req);
		if let Ok(_) = p.verify(*pub_key) {
			depot.insert("req", p.req_json);
		} else {
			res.render(StatusCode::BAD_REQUEST);
		}
	} else {
		res.render(StatusCode::UNAUTHORIZED);
	}
}

#[handler]
pub async fn force_auth(req: &mut Request, depot: &mut Depot, res: &mut Response) {
	if let Ok(_auth_data) = update_auth(req, depot).await {
	} else {
		res.render(StatusCode::UNAUTHORIZED);
	}
}


#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AuthToken {
	pub origin: String,
	pub scope: String,
	pub valid_secs: i64,
	pub created: i64,
	pub proof: String,
	pub recid: u8,
}

impl AuthToken {
	pub fn new(
		secret_key: SecretKey,
		origin: &str,
		scope: &str,
		valid_secs: i64,
		delay_secs: i64,
	) -> Result<Self, &'static str> {
		let created = (Utc::now() - Duration::seconds(delay_secs)).timestamp();
		let msg_hash = Self::hash_msg(origin, scope, valid_secs, created);
		let (recid, proof) = SECP256K1
			.sign_ecdsa_recoverable(&msg_hash, &secret_key)
			.serialize_compact();
		// let proof = secret_key.sign_ecdsa(msg_hash);

		Ok(Self {
			origin: origin.to_string(),
			scope: scope.to_string(),
			valid_secs,
			created,
			proof: hex::encode(proof),
			recid: recid.to_i32() as u8,
		})
	}

	fn from(s: String) -> Result<AuthToken, serde_json::Error> {
		let a = serde_json::from_str::<AuthToken>(&s);
		let b = a.unwrap();
		Ok(Self {
			origin: b.origin,
			scope: b.scope,
			valid_secs: b.valid_secs,
			created: b.created,
			proof: b.proof,
			recid: b.recid,
		})
	}

	pub fn expired(&mut self) -> bool {
		(self.created + self.valid_secs) < Utc::now().timestamp()
	}

	pub fn hash_msg(origin: &str, scope: &str, valid_secs: i64, created: i64) -> Message {
		let msg = &[
			origin.as_bytes(),
			scope.as_bytes(),
			&valid_secs.to_be_bytes(),
			&created.to_be_bytes(),
		]
		.concat();
		// let msg_hash = Message::from_hashed_data::<sha256::Hash>(msg);
		let digest: [u8; 32] = Keccak256::new_with_prefix(msg)
			.finalize()
			.as_slice()
			.try_into()
			.expect("Wrong length");
		Message::from_slice(&digest).unwrap()
	}

	pub fn from_header_value(hv: &header::HeaderValue) -> Result<Self, &'static str> {
		Ok(serde_json::from_str::<AuthToken>(hv.to_str().unwrap()).unwrap())
	}

	pub fn recover(self: &mut AuthToken) -> Result<PublicKey, secp256k1::Error> {
		let msg_hash = Self::hash_msg(&self.origin, &self.scope, self.valid_secs, self.created);
		let sig = hex::decode(self.proof.clone()).unwrap();
		let signr = RecoverableSignature::from_compact(
			&sig,
			RecoveryId::from_i32(self.recid as i32).unwrap(),
		);
		SECP256K1.recover_ecdsa(&msg_hash, &signr.unwrap())
	}
}

impl From<&header::HeaderValue> for AuthToken {
	fn from(hv: &header::HeaderValue) -> AuthToken {
		serde_json::from_str::<AuthToken>(hv.to_str().unwrap()).unwrap()
	}
}

impl From<String> for AuthToken {
	fn from(hv: String) -> AuthToken {
		serde_json::from_str::<AuthToken>(&hv).unwrap()
	}
}

impl Into<header::HeaderValue> for AuthToken {
	fn into(self) -> header::HeaderValue {
		let s = serde_json::to_string(&self).unwrap();
		header::HeaderValue::from_str(&["COMN", &s].join(" ")).unwrap()
	}
}
