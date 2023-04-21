async fn start_daemon() {

	//TODO warmup logger

	//TODO start OnionGateway

	//TODO start CredentialsHandler

	//TODO start BoardPublisher
	
	//TODO start RelayHandler

}

#[tokio::main]
pub async fn main() {
	start_daemon().await;
}
