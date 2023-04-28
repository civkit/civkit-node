
mod boardmanager;
mod anchormanager;

use crate::boardmanager::BoardManager;
use civkit::anchormanager::AnchorManager;
use civkit::credentialgateway::CredentialGateway;
use civkit::kindprocessor::KindProcessor;
use civkit::nodesigner::NodeSigner;

use std::sync::Arc;

async fn start_daemon() {

	//TODO warmup logger

	//TODO start OnionGateway
	
	//TODO start RelayHandler

	let credential_gateway = Arc::new(CredentialGateway::new());

	let kind_processor = Arc::new(KindProcessor::new());

	let node_signer = Arc::new(NodeSigner::new());

	let anchor_manager = Arc::new(AnchorManager::new());

	BoardManager::new(credential_gateway, node_signer, anchor_manager, kind_processor);
}

#[tokio::main]
pub async fn main() {
	start_daemon().await;
}
