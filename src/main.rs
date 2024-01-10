use comn_broker::{app, serve};

#[tokio::main]
async fn main() {
	tracing_subscriber::fmt().init();

	app().await;
	serve().await;
}
