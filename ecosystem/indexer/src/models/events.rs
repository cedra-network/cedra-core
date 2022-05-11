// Copyright (c) Aptos
// SPDX-License-Identifier: Apache-2.0

use crate::{models::transactions::Transaction, schema::events};
use aptos_rest_client::aptos_api_types::Event as APIEvent;
use serde::Serialize;

#[derive(Associations, Debug, Identifiable, Insertable, Queryable, Serialize)]
#[diesel(table_name = "events")]
#[belongs_to(Transaction, foreign_key = "transaction_hash")]
#[primary_key(key, sequence_number)]
pub struct Event {
    pub transaction_hash: String,
    pub key: String,
    pub sequence_number: i64,
    #[diesel(column_name = type)]
    pub type_: String,
    pub data: serde_json::Value,

    // Default time columns
    pub inserted_at: chrono::NaiveDateTime,
}

impl Event {
    pub fn from_event(transaction_hash: String, event: &APIEvent) -> Self {
        let mut _data = event.data.clone();
        let from_message = serde_json::to_string(&_data["from_message"]).unwrap();
        let to_message = serde_json::to_string(&_data["from_message"]).unwrap();

        _data["from_message"] = serde_json::Value::from(
            from_message
                .trim_matches(char::from(0))
                .replace("\\u0000", ""),
        );
        _data["to_message"] = serde_json::Value::from(
            to_message
                .trim_matches(char::from(0))
                .replace("\\u0000", ""),
        );

        Event {
            transaction_hash,
            key: event.key.to_string(),
            sequence_number: event.sequence_number.0 as i64,
            type_: event.typ.to_string(),
            data: _data,
            inserted_at: chrono::Utc::now().naive_utc(),
        }
    }

    pub fn from_events(transaction_hash: String, events: &[APIEvent]) -> Option<Vec<Self>> {
        if events.is_empty() {
            return None;
        }
        Some(
            events
                .iter()
                .map(|event| Self::from_event(transaction_hash.clone(), event))
                .collect::<Vec<EventModel>>(),
        )
    }
}

// Prevent conflicts with other things named `Event`
pub type EventModel = Event;
