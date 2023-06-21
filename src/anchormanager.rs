// This file is Copyright its original authors, visible in version control
// // history.
// //
// // This file is licensed under the Apache License, Version 2.0 <LICENSE-APACHE
// // or http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// // <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your option.
// // You may not use this file except in accordance with one or both of these
// // licenses.
//
// //! The top-level component of a Civ Kit node responsible to sanitize and
// //! order trade kinds, counter-sign and anchor them and dispatch them to
// //! clients according to requests.

/// A component to commit a note in the Bitcoin chain by relying on a notary service.

use std::sync::Arc;
use std::sync::Mutex;

// use civkit::events;
use civkit::events::{MessageSendKind, MessageSendKindProvider};
use civkit::kindprocessor::KindProcessor;

pub struct AnchorManager {}

impl AnchorManager {
	pub fn new() -> Self {
		AnchorManager {}
	}

	fn commit_note(&self) {}
}

impl MessageSendKindProvider for AnchorManager {
       fn get_and_clear_pending_kinds(&self) -> Vec<MessageSendKind> {
               return (vec![])
       }
