use comn_broker;
use comn_broker::auth_token::AuthToken;
use comn_broker::{db::set_db, utils::default_env};
use salvo::http::header::HeaderValue;
use secp256k1::hashes::sha256;
use secp256k1::{PublicKey, SecretKey};
use sqlx::PgPool;

pub async fn setup(pool: PgPool) {
	default_env("BIND_ADDR", "127.0.0.1:5800");
	set_db(pool);
}

pub fn get_keys(s: &str) -> (SecretKey, PublicKey) {
	let secret_key = SecretKey::from_hashed_data::<sha256::Hash>(s.as_bytes());
	let public_key = PublicKey::from_secret_key_global(&secret_key);
	(secret_key, public_key)
}

pub fn make_auth_header(
	h: &str,
	origin: &str,
	scope: &str,
	valid_secs: i64,
	delay_secs: i64,
) -> HeaderValue {
	let (secret_key, _public_key) = get_keys(h);
	// println!("test_get_crate_item {:X?} {:X?} {}", secret_key.display_secret(),
	// 		 hex::encode(public_key.serialize()), mem::size_of::<PublicKey>());
	let auth_tok = AuthToken::new(secret_key, origin, scope, valid_secs, delay_secs).unwrap();
	// println!("{:?}", auth_tok);
	auth_tok.into()
}
