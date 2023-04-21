// This file is Copyright its original authors, visible in version control
// history.
//
// This file is licensed under the Apache License, Version 2.0 <LICENSE-APACHE
// or http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your option.
// You may not use this file except in accordance with one or both of these
// licenses.

//! An interface to sanitize and enforce service policy on the received notes.

use std::sync::Mutex;

pub struct NoteProcessor {
	note_counters: Mutex<u64>,
}

impl NoteProcessor {
	pub fn new() -> Self {
		NoteProcessor {
			note_counters: Mutex::new(0),
		}
	}

	pub fn process_note(&self, note: Vec<u8>) -> bool {
		println!("Received a note !");

		if let Ok(mut note_counters_lock) = self.note_counters.lock() {
			(*note_counters_lock) += 1;
		}

		return true;
	}

	pub fn note_stats(&self) -> u64 {
		let mut notes = 0;
		if let Ok(note_counters_lock) = self.note_counters.lock() {
			notes = *note_counters_lock;
		}
		return notes;
	}
}
