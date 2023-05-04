mod boardmanager;
mod anchormanager;

use crate::boardmanager::BoardManager;
use civkit::anchormanager::AnchorManager;
use civkit::credentialgateway::CredentialGateway;
use civkit::kindprocessor::KindProcessor;
use civkit::nodesigner::NodeSigner;

use boardctrl::board_ctrl_server::{BoardCtrl, BoardCtrlServer};
use boardctrl::{PingRequest, PongRequest};

use std::sync::Arc;

use tonic::{transport::Server, Request, Response, Status};

pub mod boardctrl {
	tonic::include_proto!("boardctrl");
}

#[tonic::async_trait]
impl BoardCtrl for BoardManager {
	async fn ping_handle(&self, request: Request<PingRequest>) -> Result<Response<PongRequest>, Status> {
		let pong = PongRequest {
			name: format!("{}", request.into_inner().name).into(),
		};

		Ok(Response::new(pong))
	}
}

#[tokio::main]
pub async fn main() -> Result<(), Box<dyn std::error::Error>> {
	//TODO warmup logger

	//TODO start OnionGateway
	
	//TODO start RelayHandler

	let credential_gateway = Arc::new(CredentialGateway::new());

	let kind_processor = Arc::new(KindProcessor::new());

	let node_signer = Arc::new(NodeSigner::new());

	let anchor_manager = Arc::new(AnchorManager::new());

	let board_manager = BoardManager::new(credential_gateway, node_signer, anchor_manager, kind_processor);

	let addr = "[::1]:50001".parse()?;

	Server::builder()
		.add_service(BoardCtrlServer::new(board_manager))
		.serve(addr)
		.await?;

    	Ok(())
}
