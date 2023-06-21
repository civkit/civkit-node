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

use nostr::{Event, SubscriptionId};

#[derive(Clone, Debug)]
pub enum ClientEvents {
	TextNote { event: Event },
	Server { cmd: ServerCmd },
	EndOfStoredEvents { client_id: u64, sub_id: SubscriptionId },
	RelayNotice { message: String },
	SubscribedEvent { client_id: u64, sub_id: SubscriptionId, event: Event },
}

#[derive(Clone, Debug)]
pub enum ServerCmd {
	DisconnectClient { client_id: u64 },
}

pub trait EventsProvider {
	fn get_and_clear_pending_events(&self) -> Vec<ClientEvents>;
}
