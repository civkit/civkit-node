// This file is Copyright its original authors, visible in version control
// history.
//
// This file is licensed under the Apache License, Version 2.0 <LICENSE-APACHE
// or http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your option.
// You may not use this file except in accordance with one or both of these
// licenses.

use adminctrl::admin_ctrl_client::AdminCtrlClient;
//TODO: simplify by using prefix
use adminctrl::{PingRequest, PongRequest, ShutdownRequest, ShutdownReply, SendNote, ReceivedNote, ListClientRequest, ListSubscriptionRequest, PeerConnectionRequest, DisconnectClientRequest, SendNotice, SendOffer, SendInvoice, ListDbEventsRequest, ListDbClientsRequest, ListDbClientsReply, CheckChainStateRequest, CheckChainStateReply, GenerateTxInclusionProofRequest, GenerateTxInclusionProofReply};

use std::env;
use std::process;

use bitcoin_hashes::Hash;
use bitcoin_hashes::sha256;
use bitcoin::secp256k1::{PublicKey, Secp256k1, SecretKey};
use bitcoin::KeyPair;
use lightning::offers::offer::{Offer, OfferBuilder, Quantity};

use lightning::ln::PaymentSecret;
use lightning_invoice::{Currency, InvoiceBuilder};

use lightning::util::ser::Writeable;

use std::time::SystemTime;

use tokio::time::Duration;

use clap::{Parser, Subcommand};

pub mod adminctrl {
	tonic::include_proto!("adminctrl");
}

#[derive(Parser, Debug)]
struct Cli {
	/// The port of the connected server
	#[clap(short, long, default_value = "50031")]
	port: String,

	#[clap(subcommand)]
	command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
	/// Send a ping message
	Ping,
	/// Shutdown the connected CivKit node
	Shutdown,
	/// Send a demo NIP-01 EVENT kind 1 to all the connected clients
	Publishtextnote,
	/// List information about connected clients
	Listclients,
	/// List information about subscriptions [TODO]
	Listsubscriptions,
	/// Connect to a BOLT8 peer on local port
	Connectpeer {
		/// The port number for the peer
		peer_local_port: String,
	},
	/// Disconnect from a client [TODO]
	Disconnectclient,
	/// Send a demo NIP-01 NOTICE to all the connected clients
	Publishnotice,
	/// Send a BOLT12 offers to all the connected clients
	Publishoffer {
		/// The BOLT12 offer to be announced
		offer: String,
	},
	Publishinvoice {
		invoice: String,
	},
	/// List DB entries
	ListDbEvents,
	/// List DB clients entries
	ListDbClients,
	/// Check chain state is available
	CheckChainState,
	/// Generate a merkle block (header + merkle branch) for the target txid
	GenerateTxInclusionProof {
		txid: Vec<u8>,
	}
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {

	let cli = Cli::parse();

	let mut client = AdminCtrlClient::connect(format!("http://[::1]:{}", cli.port)).await?;

	match cli.command {
		Command::Ping => {
			let request = tonic::Request::new(PingRequest {
				name: "PING".into(),
			});

			let response = client.ping_handle(request).await?;

			println!("[CIVKIT-CLI] {}", response.into_inner().name);
		}
		Command::Shutdown => {
			let request = tonic::Request::new(ShutdownRequest {});

			let response = client.shutdown_handle(request).await?;
		}
		Command::Publishtextnote => {
			let request = tonic::Request::new(SendNote {
				content: String::from("Hello World !"),
			});

			let response = client.publish_text_note(request).await?;

			println!("[CIVKIT-CLI] {}", response.into_inner().name);
		}
		Command::Listclients => {
			let request = tonic::Request::new(ListClientRequest {});

			let response = client.list_clients(request).await?;

			println!("[CIVKIT-CLI] clients {:#?}", response.into_inner().clients);
		}
		Command::Listsubscriptions => {
			let request = tonic::Request::new(ListSubscriptionRequest {});

			let response = client.list_subscriptions(request).await?;

			println!("[CIVKIT-CLI] subscriptions {}", response.into_inner().subscriptions);
		}
		Command::Connectpeer { peer_local_port } => {
			let request = tonic::Request::new(PeerConnectionRequest {
				local_port: u64::from_str_radix(&peer_local_port, 10).unwrap()
			});
			let response = client.connect_peer(request).await?;
		}
		Command::Disconnectclient => {
			//TODO: take real client id from input
			let request = tonic::Request::new(DisconnectClientRequest {
				client_id: 0,
			});

			let _response = client.disconnect_client(request).await?;
		}
		Command::Publishnotice => {
			let request = tonic::Request::new(SendNotice {
				info_message: String::from("This is a notice"),
			});

			let response = client.publish_notice(request).await?;
		}
		Command::Publishoffer { offer } => {
			//TODO: take real offer from input
			let secp_ctx = Secp256k1::new();
			let keys = KeyPair::from_secret_key(&secp_ctx, &SecretKey::from_slice(&[42;32]).unwrap());
			let pubkey = PublicKey::from(keys);

			let expiration = SystemTime::now() + Duration::from_secs(24 * 60 * 60);

			let offer = OfferBuilder::new("naira".to_string(), pubkey)
				.amount_msats(10_000)
				.supported_quantity(Quantity::Unbounded)
				.absolute_expiry(expiration.duration_since(SystemTime::UNIX_EPOCH).unwrap())
				.issuer("Foo Bar".to_string())
				.build().unwrap();
			let mut bytes = Vec::new();
			offer.write(&mut bytes).unwrap();
			let request = tonic::Request::new(SendOffer {
				offer: bytes,
			});

			let response = client.publish_offer(request).await?;
		}
		Command::Publishinvoice { invoice } => {
			//TODO: tale real invoice from input
			let secret_key = SecretKey::from_slice(&[42;32]).unwrap();
	
			let payment_hash = sha256::Hash::from_slice(&[0; 32][..]).unwrap();
			let payment_secret = PaymentSecret([42u8;32]);

			let invoice = InvoiceBuilder::new(Currency::Bitcoin)
				.description("Here a trade invoice!".into())
				.payment_hash(payment_hash)
				.payment_secret(payment_secret)
				.current_timestamp()
				.min_final_cltv_expiry_delta(144)
				.build_signed(|payment_hash| {
					Secp256k1::new().sign_ecdsa_recoverable(payment_hash, &secret_key)
				})
				.unwrap();

			let request = tonic::Request::new(SendInvoice {
				invoice: invoice.to_string()
			});

			let response = client.publish_invoice(request).await?;
		}
		Command::ListDbEvents => {
			let request = tonic::Request::new(ListDbEventsRequest {});

			let _response = client.list_db_events(request).await?;
		}
		Command::ListDbClients => {
			let request = tonic::Request::new(ListDbClientsRequest {});

			let _response = client.list_db_clients(request).await?;
		}
		Command::CheckChainState => {
			let request = tonic::Request::new(CheckChainStateRequest {});

			let _response = client.check_chain_state(request).await?;
		}
		Command::GenerateTxInclusionProof { txid } => {
			let request = tonic::Request::new(GenerateTxInclusionProofRequest {
				txid: txid.to_vec(),
			});

			let _response = client.generate_tx_inclusion_proof(request).await?;
		}
	}
	Ok(())
}
