use boardctrl::board_ctrl_client::BoardCtrlClient;
use boardctrl::{PingRequest, PongRequest};

pub mod boardctrl {
	tonic::include_proto!("boardctrl");
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
	let mut client = BoardCtrlClient::connect("http://[::1]:50001").await?;

	let request = tonic::Request::new(PingRequest {
		name: "civkit - client".into(),
	});

	let response = client.ping_handle(request).await?;

	println!("RESPONSE={:?}", response);

	Ok(())
}
