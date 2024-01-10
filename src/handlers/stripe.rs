use stripe::{EventObject, EventType, Webhook}; 
use salvo::prelude::{handler, Request, Response, Depot};
use serde::{Deserialize, Serialize};
use salvo::http::{StatusCode};
use std::str;
use crate::{
    comn_addr::ComnAddr,
	update::coin::{Transaction, TransactionErr},
};
use rand::{thread_rng, Rng};
use rand::distributions::Alphanumeric;
// let client = 
// 	stripe::Client::new(
// 		"sk_test_51N8zw0E899ffxgHIixCZiHY5Zzd56b6zwLT7e6r1Z0BtaUMYFcBloiiKfdq7z3OdoxjbZCW2ZCiWIrx9nLomsouv00nEw4jcu6"
// 	);

#[derive(Serialize, Deserialize, Debug)]
pub struct Payload {
    pub contents: String,
}

#[handler]
pub async fn check_sign(req: &mut Request, depot: &mut Depot) {
	let header = req.headers().get("stripe-signature").unwrap();
	let signature = header.to_str().unwrap();
	depot.insert("stripe_signature", signature.to_string());
}

#[handler]
pub async fn stripe_webhook(req: &mut Request, res: &mut Response, depot: &mut Depot) {
	let payload_bytes = req.payload().await.unwrap();
	let payload = str::from_utf8(payload_bytes).unwrap();

	let signature = depot.get::<String>("stripe_signature").unwrap();

	if let Ok(event) = Webhook::construct_event(
        payload,
        &signature,
        "whsec_e816dd536f7e6229e179fa9564dea1807582a4e94065b91ae943d39c9cacba55",
    ) {
        match event.event_type {
            EventType::CheckoutSessionCompleted => {
                if let EventObject::CheckoutSession(session) = event.data.object {
                    println!("session {:?}", session);
                    let token_amount: u64 = ((session.amount_subtotal.unwrap())/100).try_into().unwrap();
                    println!("amount {:?}", token_amount);
                    if let Ok(dd) = serde_json::from_str::<CustomField>(&payload) {
                        if let Some(input) = dd.data.object.custom_fields.iter().find(|&x| x.key == "addr".to_string()) {
                            let mut receiver_addr = input.text.value.clone().trim().to_string();
                            if receiver_addr.starts_with('≈') {
                            } else {
                                receiver_addr = "≈".to_owned()+&receiver_addr;
                            }
                            let nonce: String = thread_rng()
                                .sample_iter(&Alphanumeric)
                                .take(32)
                                .map(char::from)
                                .collect();
                            let transaction = Transaction {
                                amount: token_amount,
                                receiver: ComnAddr::new(&receiver_addr).unwrap(),
                                sender: ComnAddr::new("≈6D").unwrap(),
                                comment: Some("Bought from Opus Website.".to_string()),
                                nonce: nonce,
                            };
                            match transaction.transfer_coins().await {
                                Ok(_) => res.render(StatusCode::OK), 
                                Err(TransactionErr::AlreadyReported) => res.render(StatusCode::ALREADY_REPORTED),
                                Err(TransactionErr::LowAmount) | Err(TransactionErr::BadData) =>
                                    res.render(StatusCode::INTERNAL_SERVER_ERROR),
                            }
                        } else {
                            res.render(StatusCode::BAD_REQUEST);
                        }
                    }
                }
            }
            _ => res.render(StatusCode::METHOD_NOT_ALLOWED),
        }
    } else {
        res.render(StatusCode::BAD_REQUEST);
    }
}


#[derive(Serialize, Deserialize, Debug)]
pub struct CustomField {
    pub data: DataL2,
}
#[derive(Serialize, Deserialize, Debug)]
pub struct DataL2 {
    pub object: ObjectL3,
}
#[derive(Serialize, Deserialize, Debug)]
pub struct ObjectL3 {
    pub custom_fields: Vec<CustomFieldsL4>
}
#[derive(Serialize, Deserialize, Debug)]
pub struct CustomFieldsL4 {
    pub key: String,
    pub text: TextL5,
}
#[derive(Serialize, Deserialize, Debug)]
pub struct TextL5 {
    pub value: String,
}