// Copyright (c) Aptos
// SPDX-License-Identifier: Apache-2.0

use serde::{Deserialize, Serialize};
use serde_json::Result;
use serde_json::Value;
use std::collections::HashMap;

use crate::util;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PropertyValue {
    value: String,
    typ: String,
}

pub fn create_property_value(typ: String, value: String) -> Result<PropertyValue> {
    Ok(PropertyValue {
        value: util::convert_bcs_hex(typ.clone(), value.clone()).unwrap_or(value),
        typ,
    })
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PropertyMap {
    data: HashMap<String, PropertyValue>,
}

impl PropertyMap {
    /// Deserializes PropertyValue from bcs encoded json
    pub fn from_bsc_encode_str(val: Value) -> Option<Value> {
        let mut pm = PropertyMap {
            data: HashMap::new(),
        };
        let records: &Vec<Value> = val.get("map")?.get("data")?.as_array()?;
        for entry in records {
            let key = entry.get("key")?.as_str()?;
            let val = entry.get("value")?.get("value")?.as_str()?;
            let typ = entry.get("value")?.get("type")?.as_str()?;
            let pv = create_property_value(typ.to_string(), val.to_string()).ok()?;
            pm.data.insert(key.to_string(), pv);
        }
        Some(Self::to_flat_json(pm))
    }

    /// Flattens PropertyMap which can't be easily consumable by downstream.
    /// For example: Object {"data": Object {"creation_time_sec": Object {"value": String("1666125588")}}}
    /// becomes Object {"creation_time_sec": "1666125588"}
    fn to_flat_json(val: PropertyMap) -> Value {
        let mut map = HashMap::new();
        for (k, v) in val.data {
            map.insert(k, v.value);
        }
        serde_json::to_value(map).unwrap()
    }
}
