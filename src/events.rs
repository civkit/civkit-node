// This file is Copyright its original authors, visible in version control
// history.
//
// This file is licensed under the Apache License, Version 2.0 <LICENSE-APACHE
// or http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your option.
// You may not use this file except in accordance with one or both of these
// licenses.

//! Internal events used to exchange information between ServiceManager and
//! ClientHandler.

use crate::NostrClient;

use nostr::{Event, EventId, SubscriptionId};

use tokio::sync::oneshot;

#[derive(Debug)]
pub enum ClientEvents {
	TextNote { event: Event },
	Server { cmd: ServerCmd },
	OrderNote { order: Event },
	StoredEvent { client_id: u64, events: Vec<Event> },
	EndOfStoredEvents { client_id: u64, sub_id: SubscriptionId },
	RelayNotice { client_id: u64, message: String },
	SubscribedEvent { client_id: u64, sub_id: SubscriptionId, event: Event },
	OkEvent { event_id: EventId, ret: bool, msg: Option<String> },
}

#[derive(Debug)]
pub enum ServerCmd {
	DisconnectClient { client_id: u64 },
	GetClients { respond_to: oneshot::Sender<Vec<NostrClient>> }
}

pub trait EventsProvider {
	fn get_and_clear_pending_events(&self) -> Vec<ClientEvents>;
}
