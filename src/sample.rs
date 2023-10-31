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
use bitcoin::Txid;
use bitcoin::hashes::{Hash, sha256, HashEngine};

use staking_credentials::common::utils::{Credentials, Proof};
use staking_credentials::common::msgs::{CredentialAuthenticationPayload, ServiceDeliveranceRequest};

use nostr::{RelayMessage, EventBuilder, Metadata, Keys, ClientMessage, Kind, Filter, SubscriptionId, Timestamp, Tag};

use url::Url;

use clap::{Arg, Command};

use futures_channel;
use futures_util::{future, pin_mut, StreamExt, TryStreamExt};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message, tungstenite::error::Error};

use std::str::FromStr;

const CLIENT_SECRET_KEY: [u8; 32] = [ 59, 148, 11, 85, 134, 130, 61, 253, 2, 174, 59, 70, 27, 180, 51, 107, 94, 203, 174, 253, 102, 39, 170, 146, 46, 252, 4, 143, 236, 12, 136, 28];

struct CredentialsHolder {
	state: Vec<([u8; 32], Signature)>,
}

impl CredentialsHolder {
	fn new() -> Self {
		CredentialsHolder {
			state: Vec::new(),
		}
	}
}

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

        match respond(&line, &tx, &client_keys) {
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
	    	.args([Arg::new("content").help("the order type (either bolt11 or bolt12)").required(true)])
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
	    	.args([Arg::new("txid").help("The transaction id").required(true)])
		.help_template(APPLET_TEMPLATE)
		.about("Submit a credential proof to the relay"),
	)
        .subcommand(
            Command::new("shutdown")
                .help_template(APPLET_TEMPLATE)
                .about("Shutdown the REPL"),
        )
}

fn respond(
    line: &str,
    tx: &futures_channel::mpsc::UnboundedSender<Message>,
    client_keys: &Keys,
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

	    let credentials = vec![];
	    let signatures = vec![];
	    let service_id = 0;

	    let secp = Secp256k1::new();
	    let msg = b"";

	    let hash_msg = sha256::Hash::hash(msg);
	    let msg = SecpMessage::from_slice(hash_msg.as_ref()).unwrap();
	    let seckey = SecretKey::from_slice(&CLIENT_SECRET_KEY).unwrap();

	    let commitment_sig = secp.sign_ecdsa(&msg, &seckey);

	    let mut service_deliverance_request = ServiceDeliveranceRequest::new(credentials, signatures, service_id, commitment_sig);
	    //TODO: serialize service_deliverance_request

	    let empty_content = String::new();
	    if let Ok(kind_3251_event) =
		EventBuilder::new_service_deliverance_request(&empty_content, &[]).to_event(client_keys)
	    {
	        let client_message = ClientMessage::new_event(kind_3251_event);
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
            let serialized_message = client_message.as_json();
            tx.unbounded_send(Message::text(serialized_message))
                .unwrap();
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
	    let txid_parse: Option<&String> = matches.get_one("txid");
	    let txid_str = txid_parse.unwrap();

	    let bytes = txid_str.as_bytes();
	    let mut enc = Txid::engine();
	    enc.input(&bytes);
	    let txid = Txid::from_engine(enc);

	    let proof = Proof::Txid(txid);
	    //TODO: get credentials from sample local holder state
	    let credentials = vec![Credentials([16;32])];

	    let credential_authentication = CredentialAuthenticationPayload::new(proof, credentials);

	    //TODO: credential_authentication.serialize()
	    let tags = &[
		Tag::Credential(vec![]),
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
        _ => {
            println!("Unknown command");
            return Ok(true);
        }
    }

    Ok(false)
}

async fn poll_for_server_output(mut rx: futures_channel::mpsc::UnboundedReceiver<Message>) {

    loop {
        if let Ok(message) = rx.try_next() {
			let msg = message.unwrap();
                let msg_json = String::from_utf8(msg.into()).unwrap();
                //println!("Received message {}", msg_json);
                if let Ok(relay_msg) = RelayMessage::from_json(msg_json) {
                    match relay_msg {
			RelayMessage::Event { subscription_id, event } => {
			    //TODO: NIP 01: `EVENT` messages MUST be sent only with a subscriptionID related to a subscription previously initiated by the client (using the `REQ` message above)`
			    let display_board_order = if event.kind == Kind::Order { true } else { false };
			    println!("\n[EVENT] {}  {}", if display_board_order { "new trade offer: " } else { "" }, event.content);
			    io::stdout().flush().unwrap();
			},
                        RelayMessage::Notice { message } => {
                            println!("\n[NOTICE] {}", message);
                            print!("> ");
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

    let connect_addr = env::args().nth(1).unwrap_or_else(|| "50021".to_string());

    let addr = format!("ws://[::1]:50021");

    let url = url::Url::parse(&addr).unwrap();

    // Init client state
    let keys = Keys::generate();

    let credential_state = CredentialsHolder::new();

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
