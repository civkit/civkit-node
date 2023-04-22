
mod boardmanager;
mod anchormanager;

use crate::boardmanager::BoardManager;
use civkit::anchormanager::AnchorManager;

use std::sync::Arc;

async fn start_daemon() {

	//TODO warmup logger

	//TODO start OnionGateway

	//TODO start CredentialsHandler

	//TODO start BoardPublisher
	
	//TODO start RelayHandler
	let anchor_manager = Arc::new(AnchorManager::new());

	BoardManager::new(anchor_manager);
}

#[tokio::main]
pub async fn main() {
	start_daemon().await;
}
