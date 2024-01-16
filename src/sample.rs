// This file is Copyright its original authors, visible in version control
// history.
//
// This file is licensed under the Apache License, Version 2.0 <LICENSE-APACHE
// or http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your option.
// You may not use this file except in accordance with one or both of these
// licenses.

use std::env;
use std::io;
use std::io::Write;
use std::process;

use bitcoin::secp256k1::{PublicKey, SecretKey, Secp256k1, Signature};
use bitcoin::secp256k1::Message as SecpMessage;
use bitcoin::blockdata::transaction::Transaction;
use bitcoin::{MerkleBlock, Txid};
use bitcoin::hashes::{Hash, sha256, HashEngine};
use bitcoin_hashes::hex::FromHex;
use bitcoin::secp256k1;

use staking_credentials::common::utils::{Credentials, Proof};
use staking_credentials::common::msgs::{CredentialAuthenticationPayload, CredentialAuthenticationResult, Encodable, Decodable, ServiceDeliveranceRequest, ToHex, CredentialPolicy, ServicePolicy};

use nostr::{RelayMessage, EventBuilder, Metadata, Keys, ClientMessage, Kind, Filter, SubscriptionId, Timestamp, Tag};

use url::Url;

use clap::{Arg, Command};

use futures_channel;
use futures_util::{future, pin_mut, StreamExt, TryStreamExt};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message, tungstenite::error::Error};

use std::str::FromStr;
use crate::civkitservice::civkit_service_client::CivkitServiceClient;

pub mod civkitservice {
	tonic::include_proto!("civkitservice");
}

use std::collections::HashMap;
use std::sync::Mutex;
use std::ops::Deref;

const CLIENT_SECRET_KEY: [u8; 32] = [ 59, 148, 11, 85, 134, 130, 61, 253, 2, 174, 59, 70, 27, 180, 51, 107, 94, 203, 174, 253, 102, 39, 170, 146, 46, 252, 4, 143, 236, 12, 136, 28];

// Debug purpose only
const GATEWAY_SECRET_KEY: [u8; 32] = [ 57, 149, 12, 84, 135, 129, 62, 252, 3, 173, 60, 69, 28, 179, 52, 106, 95, 202, 175, 252, 103, 40, 169, 147, 45, 253, 5, 142, 235, 13, 135, 29];

const DEFAULT_CREDENTIAL: u8 = 1;

macro_rules! check_credentials_sigs_order {
	($credentials: expr, $signatures: expr, $pubkey: expr) => {
		{
			let secp_ctx = Secp256k1::new();

			let mut index = 0;
	
			for signed_credentials in $credentials.iter().zip($signatures.iter()) {

				let credential_bytes = signed_credentials.0.serialize();

				if let Ok(msg) = secp256k1::Message::from_slice(&credential_bytes[..]) {
					let ret = secp_ctx.verify(&msg, &signed_credentials.1, &$pubkey);
					assert!(ret.is_ok(), "sig check fails at {}", index);
				}
				index += 1;
			}
		}
	}
}

struct Service {
	pubkey: PublicKey,
	credential_policy: CredentialPolicy,
	service_policy: ServicePolicy,
}

struct CredentialsHolder {
	//TODO: add source of randomness ?
	state: (Vec<Credentials>, Vec<Signature>),
	service_pubkey_to_policy: Vec<(PublicKey, String)>, //TODO: add PolicyMessage
	registered_services: Vec<Service>,
}

impl CredentialsHolder {
	fn new() -> Self {
		CredentialsHolder {
			state: (Vec::new(), Vec::new()),
			service_pubkey_to_policy: Vec::new(),
			registered_services: Vec::new(),
		}
	}

	fn generate_credentials(&mut self, num_credentials: u32) -> Vec<Credentials> {
		//TODO: add source of randomness
		let mut credentials = Vec::new();

		for num in 0..num_credentials {
			let c = Credentials([0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31]);
			credentials.push(c);
		}
		credentials
	}

	fn check_credential(&mut self, service_pubkey: &PublicKey) -> bool {
		for service in &self.service_pubkey_to_policy {
			if *service_pubkey == service.0 {		
				//TODO: for now assume default credential is 1
				//and all credentials are signed by same key.
				if self.state.0.len() == DEFAULT_CREDENTIAL as usize {
					return true;
				}
			}
		}
		//TODO: for now return always true, implement CredentialGateway self-announcement of its pubkey.
		return true;
	}

	fn register_new_service(&mut self, new_service: Service) {
		self.registered_services.push(new_service);
	}

	fn store_signatures(&mut self, mut signatures: Vec<Signature>) {
		self.state.1.append(&mut signatures);
		println!("debug: stored signatures {}", self.state.1.len());
	}

	fn store_credentials(&mut self, mut credentials: Vec<Credentials>) {
		self.state.0.append(&mut credentials);
		println!("debug: stored credentials {}", self.state.0.len());
	}

	fn get_signed_credentials(&mut self, num_credential: u64) -> (Vec<Credentials>, Vec<Signature>) {
		let mut credentials = vec![];
		let mut signatures = vec![];
		println!("debug: num_credential {} signed credentials {} {}", num_credential, self.state.0.len(), self.state.1.len());
		for i in 0..num_credential {
			credentials.push(self.state.0.remove(i as usize));
			signatures.push(self.state.1.remove(i as usize));
		}
		(credentials, signatures)
	}

	fn get_credentials(&mut self) -> &Vec<Credentials> {
		&self.state.0
	}
}

static GLOBAL_HOLDER: Mutex<CredentialsHolder> = Mutex::new(CredentialsHolder {
	state: (Vec::new(), Vec::new()),
	service_pubkey_to_policy: Vec::new(),
	registered_services: Vec::new()
});

async fn poll_for_user_input(client_keys: Keys, tx: futures_channel::mpsc::UnboundedSender<Message>) {

    println!("Civkit sample startup successful. Enter \"help\" to view available commands");

    loop {
        print!("> ");
        io::stdout().flush().unwrap();
        let mut line = String::new();
        if let Err(e) = io::stdin().read_line(&mut line) {
            break println!("ERROR {}", e);
        }

        if line.len() == 0 {
            continue;
        }

        match respond(&line, &tx, &client_keys).await {
            Ok(quit) => {
                if quit {
                    process::exit(0x0100);
                }
            }
            Err(err) => {
                write!(std::io::stdout(), "{err}").expect("error: Failed to write to stdout");
                std::io::stdout()
                    .flush()
                    .expect("error: Failed to flush stdout");
            }
        }
    }
}

fn cli() -> Command {
    // strip out usage
    const PARSER_TEMPLATE: &str = "\
        {all-args}
    ";
    // strip out name/version
    const APPLET_TEMPLATE: &str = "\
        {about-with-newline}\n\
        {usage-heading}\n    {usage}\n\
        \n\
        {all-args}{after-help}\
    ";

    Command::new("Nostr Client REPL")
        .multicall(true)
        .arg_required_else_help(true)
        .subcommand_required(true)
        .help_template(PARSER_TEMPLATE)
        .subcommand(
            Command::new("sendtextnote")
                .args([Arg::new("content").help("the content of the text note").required(true)])
                .help_template(APPLET_TEMPLATE)
                .about("Send a text note to the relay"),
        )
        .subcommand(
            Command::new("setmetadata")
                .args([
					Arg::new("username").help("The client's username").required(true),
                    Arg::new("about").help("The client's about string").required(true),
                    Arg::new("picture").help("The client's picture").required(true),
                ])
                .help_template(APPLET_TEMPLATE)
                .about("Set the client's metadata"),
        )
        .subcommand(
            Command::new("recommendserver")
                .args([Arg::new("urlrelay").help("The url string of the server to recommend").required(true)])
                .help_template(APPLET_TEMPLATE)
                .about("Recommend a server to the relay"),
        )
	.subcommand(
	    Command::new("sendmarketorder")
	    	.args([
			Arg::new("content").help("the order type (either bolt11 or bolt12)").required(true),
			Arg::new("board_pubkey").help("the board pubkey").required(true),	
		])
		.help_template(APPLET_TEMPLATE)
		.about("Send a market order (kind: 32500) to the relay"),
	)
        .subcommand(
            Command::new("opensubscription")
                .args([
                    Arg::new("subscriptionid").help("The subscription id").required(true),
                    Arg::new("kinds").help("The kinds of events to subscribe to").required(true),
                    Arg::new("since").help("The timestamp to start the subscription").required(true),
                    Arg::new("until").help("The timestamp to end the subscription").required(true),
                ])
                .help_template(APPLET_TEMPLATE)
                .about("Open a subscription to the relay"),
        )
        .subcommand(
            Command::new("closesubscription")
                .args([Arg::new("subscriptionid").help("The subscription id").required(true)])
                .help_template(APPLET_TEMPLATE)
                .about("Close a subscription to the relay"),
        )
	.subcommand(
	    Command::new("submitcredentialproof")
	    	.args([Arg::new("merkle_block").help("The merkle block").required(true)])
		.help_template(APPLET_TEMPLATE)
		.about("Submit a credential proof to the relay"),
	)
	.subcommand(
	    Command::new("addservice")
	    	.args([Arg::new("publickey").help("The service public key").required(true)])
		.help_template(APPLET_TEMPLATE)
		.about("Register manually a service"),
	)
        .subcommand(
            Command::new("shutdown")
                .help_template(APPLET_TEMPLATE)
                .about("Shutdown the REPL"),
        )
        .subcommand(
            Command::new("verifyinclusionproof")
                .help_template(APPLET_TEMPLATE)
                .about("Verifies whether the most recent Merkle root from Mainstay includes the last commitment sent (the last event generated by Civkit) or not"),
        )
}

async fn respond(
    line: &str,
    tx: &futures_channel::mpsc::UnboundedSender<Message>,
    client_keys: &Keys 
) -> Result<bool, String> {
    let args = line.split_whitespace().collect::<Vec<&str>>();
    let matches = cli()
        .try_get_matches_from(args)
        .map_err(|e| e.to_string())?;

    match matches.subcommand() {
        Some(("sendtextnote", matches)) => {
            let content: Option<&String> = matches.get_one("content");
            if let Ok(kind1_event) =
                EventBuilder::new_text_note(content.unwrap(), &[]).to_event(client_keys)
            {
                let client_message = ClientMessage::new_event(kind1_event);
                let serialized_message = client_message.as_json();
                tx.unbounded_send(Message::text(serialized_message))
                    .unwrap();
            }
        }
        Some(("setmetadata", matches)) => {
            let username: Option<&String> = matches.get_one("username");
            let about: Option<&String> = matches.get_one("about");
            let picture: Option<&String> = matches.get_one("picture");
            //TODO: add picture arg
            let metadata = Metadata::new()
                .name(username.unwrap())
                .about(about.unwrap());
            if let Ok(kind0_event) = EventBuilder::set_metadata(metadata).to_event(client_keys) {
                let client_message = ClientMessage::new_event(kind0_event);
                let serialized_message = client_message.as_json();
                tx.unbounded_send(Message::text(serialized_message))
                    .unwrap();
            }
        }
        Some(("recommendserver", matches)) => {
            let urlrelay: Option<&String> = matches.get_one("urlrelay");
            if let Ok(kind2_event) =
                EventBuilder::add_recommended_relay(&Url::parse(urlrelay.unwrap()).unwrap())
                    .to_event(client_keys)
            {
                let client_message = ClientMessage::new_event(kind2_event);
                let serialized_message = client_message.as_json();
                tx.unbounded_send(Message::text(serialized_message))
                    .unwrap();
            }
        }
	Some(("sendmarketorder", matches)) => {
	    let content: Option<&String> = matches.get_one("content");
	    let board_pk: Option<&String> = matches.get_one("board_pubkey");
	    let board_pk_str = board_pk.unwrap();

	    let board_pk = PublicKey::from_str(board_pk_str).unwrap();

	    let service_id = 0;

	    let mut credentials = vec![];
	    let mut signatures = vec![];
	    {
		if let Ok(mut credential_holder_lock) = GLOBAL_HOLDER.lock() {
			if !credential_holder_lock.check_credential(&board_pk) {
				println!("Credentials are not enough");
				return Ok(true);
			}
			let signed_credentials = credential_holder_lock.get_signed_credentials(DEFAULT_CREDENTIAL as u64);
			credentials = signed_credentials.0;
			signatures = signed_credentials.1;
		}
	    }

	    #[cfg(debug_assertions)] {
		let secp_ctx = Secp256k1::new();
		let secret_key = SecretKey::from_slice(&GATEWAY_SECRET_KEY).unwrap();
		let pubkey = PublicKey::from_secret_key(&secp_ctx, &secret_key);
		check_credentials_sigs_order!(credentials, signatures, pubkey);
		println!("DEBUG SAMPLE - signature check ok");
	    }

	    let mut service_deliverance_request = ServiceDeliveranceRequest::new(credentials, signatures, service_id);

	    let mut buffer = vec![];
	    service_deliverance_request.encode(&mut buffer);
	    let service_deliverance_hex_str = buffer.to_hex();
	    let tags = &[
		Tag::Credential(service_deliverance_hex_str),
	    ];

	    if let Ok(credential_carrier) =
		EventBuilder::new_text_note("", tags).to_event(client_keys)
	    {
	        let client_message = ClientMessage::new_event(credential_carrier);
		let serialized_message = client_message.as_json();
		tx.unbounded_send(Message::text(serialized_message))
		    .unwrap();
	    }

	    if let Ok(kind_32500_event) =
	        EventBuilder::new_order_note(content.unwrap(), &[]).to_event(client_keys)
	    {

		let client_message = ClientMessage::new_event(kind_32500_event);
		let serialized_message = client_message.as_json();
		tx.unbounded_send(Message::text(serialized_message))
			.unwrap();
	    }
	}
        Some(("opensubscription", matches)) => {
            let subscriptionid: Option<&String> = matches.get_one("subscriptionid");
            let kinds_raw: Option<&String> = matches.get_one("kinds");
            let since_raw: Option<&String> = matches.get_one("since");
            let until_raw: Option<&String> = matches.get_one("until");
            let id = SubscriptionId::new(subscriptionid.unwrap());
            let kinds_vec: Vec<&str> = kinds_raw.unwrap().split(',').collect();
            let mut kinds = Vec::with_capacity(kinds_vec.len());
            for kind in kinds_vec {
                if let Ok(k) = Kind::from_str(kind) {
                    kinds.push(k);
                }
            }
            let since = Timestamp::from_str(since_raw.unwrap()).unwrap();
            let until = Timestamp::from_str(until_raw.unwrap()).unwrap();
            let filter = Filter::new().kinds(kinds).since(since).until(until);
            let client_message = ClientMessage::new_req(id, vec![filter]);
            let serialized_message = client_message.as_json(); tx.unbounded_send(Message::text(serialized_message)) .unwrap();
        }
        Some(("closesubscription", matches)) => {
            let subscriptionid: Option<&String> = matches.get_one("subscriptionid");
            let id = SubscriptionId::new(subscriptionid.unwrap());
            let client_message = ClientMessage::close(id);
            let serialized_message = client_message.as_json();
            tx.unbounded_send(Message::text(serialized_message))
                .unwrap();
        }
        Some(("shutdown", _matches)) => {
            tx.unbounded_send(Message::Close(None)).unwrap();
            tx.close_channel();
            println!("Civkit sample exiting...");
            return Ok(true);
        }
	Some(("submitcredentialproof", matches)) => {
	    let mb_parse: Option<&String> = matches.get_one("merkle_block");
	    let mb_str = mb_parse.unwrap();

	    let mb_bytes = Vec::from_hex(mb_str).unwrap();
	    let mb: MerkleBlock = bitcoin::consensus::deserialize(&mb_bytes).unwrap();
 
	    let proof = Proof::MerkleBlock(mb);

	    let mut credentials = vec![];
	    {
		if let Ok(mut credential_holder_lock) = GLOBAL_HOLDER.lock() {
			credentials = credential_holder_lock.generate_credentials(5);
			credential_holder_lock.store_credentials(credentials.clone());
		}
	    }

	    let credential_authentication = CredentialAuthenticationPayload::new(proof, credentials);
	    let mut buffer = vec![];
	    credential_authentication.encode(&mut buffer);
	    let credential_hex_str = buffer.to_hex();
	    let tags = &[
		Tag::Credential(credential_hex_str),
	    ];

	    if let Ok(credential_carrier) =
		EventBuilder::new_text_note("", tags).to_event(client_keys)
	    {
	        let client_message = ClientMessage::new_event(credential_carrier);
		let serialized_message = client_message.as_json();
		tx.unbounded_send(Message::text(serialized_message))
		    .unwrap();
	    }
	}
	Some(("verifyattestationproof", matches)) => {
            let attestation_proof: Option<&String> = matches.get_one("attestation_proof");
	    let attestation_proof_str = attestation_proof.unwrap();

	    let tags = &[
		Tag::Attestation(attestation_proof_str.as_bytes().to_vec()),
	    ];

	    if let Ok(kind_4250_event) =
		EventBuilder::new_text_note("", tags).to_event(client_keys)
	    {
		let client_message = ClientMessage::new_event(kind_4250_event);
		let serialized_message = client_message.as_json();
		tx.unbounded_send(Message::text(serialized_message))
		    .unwrap();
	    }
	}
    Some(("verifyinclusionproof", matches)) => {
        println!("verifyinclusionproof: Verifies whether the most recent Merkle root from Mainstay includes the last commitment sent (the last event generated by Civkit) or not");
        
        let request = tonic::Request::new(civkitservice::VerifyInclusionProofRequest {});

        let mut civkitd_client = CivkitServiceClient::connect(format!("http://[::1]:{}", 50031)).await;

        if let Ok(response) = civkitd_client.unwrap().verify_inclusion_proof(request).await {
            println!("verified: {:?}", response.into_inner().verified);
        }
    }
        _ => {
            println!("Unknown command");
            return Ok(false);
        }
    }

    Ok(false)
}

async fn poll_for_server_output(mut rx: futures_channel::mpsc::UnboundedReceiver<Message>) {

    loop {
        if let message = rx.next().await {
			let msg = message.unwrap();
                let msg_json = String::from_utf8(msg.into()).unwrap();
            	println!("Received message {}", msg_json);
                if let Ok(relay_msg) = RelayMessage::from_json(msg_json) {
                    match relay_msg {
			RelayMessage::Event { subscription_id, event } => {
			    if event.tags.len() == 1 {
			        let credential_hex = match &event.tags[0] {
					Tag::Credential(credential) => { credential },
					_ => { continue; }
				};
				let credential_msg_bytes = Vec::from_hex(&credential_hex).unwrap();
				let credential_authentication_result = CredentialAuthenticationResult::decode(&mut credential_msg_bytes.deref()).unwrap();
				if let Ok(mut credential_holder_lock) = GLOBAL_HOLDER.lock() {
					println!("\n[EVENT] storing {} credential signatures from a credentail result", credential_authentication_result.signatures.len());
				
	    				#[cfg(debug_assertions)] {
	    				    let secp_ctx = Secp256k1::new();
	    				    let secret_key = SecretKey::from_slice(&GATEWAY_SECRET_KEY).unwrap();
	    				    let pubkey = PublicKey::from_secret_key(&secp_ctx, &secret_key);
					    let credentials = credential_holder_lock.get_credentials();
					    println!("debug number of stored credentials {}", credentials.len());
	    				    check_credentials_sigs_order!(credentials, credential_authentication_result.signatures, pubkey);
	    				    println!("DEBUG SAMPLE - signature check ok");
	    				}

					credential_holder_lock.store_signatures(credential_authentication_result.signatures);	
				}
			    } else {
			    	//TODO: NIP 01: `EVENT` messages MUST be sent only with a subscriptionID related to a subscription previously initiated by the client (using the `REQ` message above)`
			    	let display_board_order = if event.kind == Kind::Order { true } else { false };
			    	println!("\n[EVENT] {}  {}", if display_board_order { "new trade offer: " } else { "" }, event.content);
			    	println!("> ");
			    	io::stdout().flush().unwrap();
			    }
			},
                        RelayMessage::Notice { message } => {
                            println!("\n[NOTICE] {}", message);
                            print!("> ");
			    //service_repository.register_new_service();
                            io::stdout().flush().unwrap();
			},
                        RelayMessage::EndOfStoredEvents(sub_id) => {
                            println!("\n[EOSE] {}", sub_id);
                            print!("> ");
                            io::stdout().flush().unwrap();
			},
			RelayMessage::Ok { event_id, status, message } => {
			     println!("[OK] event_id {} status {} message {}", event_id, status, message);
			     print!("> ");
                            io::stdout().flush().unwrap();
			},
			_ => { println!("Unknown server message"); }
		    }
		} else { println!("RelayMessage deserialization failure"); }
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
	// TODO add documentation for this option and use it
    let connect_addr = env::args().nth(1).unwrap_or_else(|| "50021".to_string());

    let addr = format!("ws://[::1]:50021");

    let url = url::Url::parse(&addr).unwrap();

    // Init client state
    let keys = Keys::generate();

    let (stdin_tx, stdin_rx) = futures_channel::mpsc::unbounded();
    tokio::spawn(poll_for_user_input(keys, stdin_tx));

    let (stdout_tx, stdout_rx) = futures_channel::mpsc::unbounded();
    tokio::spawn(poll_for_server_output(stdout_rx));

    let (ws_stream, _) = if let Ok(info) = connect_async(url).await {
        info
    } else {
        panic!("WebSocket connection failed !");
    };

    let (write, read) = ws_stream.split();

    let stdin_to_ws = stdin_rx.map(Ok).forward(write);
    let ws_to_stdout = read.try_for_each(|msg| {
        stdout_tx.unbounded_send(msg).unwrap();
        future::ok(())
    });

    pin_mut!(stdin_to_ws, ws_to_stdout);
    future::select(stdin_to_ws, ws_to_stdout).await;
    Ok(())
}
