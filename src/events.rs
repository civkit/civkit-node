// This file is Copyright its original authors, visible in version control
// history.
//
// This file is licensed under the Apache License, Version 2.0 <LICENSE-APACHE
// or http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your option.
// You may not use this file except in accordance with one or both of these
// licenses.

#[derive(Clone, Debug)]
pub enum MessageSendKind {
	Offers {
		sha256: [u8; 32],
	}
}

pub trait MessageSendKindProvider {
	fn get_and_clear_pending_kinds(&self) -> Vec<MessageSendKind>;
}
