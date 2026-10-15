//! How a chronolexogram holds an engine value: as JSON, without losing a
//! number.
//!
//! The engine keeps an integer as an `Int` and every other number as an
//! exact `Decimal`. Plain JSON numbers cannot carry that: the engine writes a
//! decimal to JSON as a float, so `1/3` loses its 28 digits, and it reads a
//! whole float back as an integer, so a decimal `5.0` returns as `5`. The
//! chronicle writes a decimal as `{"__decimal": "<digits>"}` instead, with
//! the digits and the scale as the engine holds them, and reads that back as
//! the same decimal. Every other value is the JSON the engine writes.

use std::collections::BTreeMap;

use regelrecht_engine::{equal_as_written, Value};
use rust_decimal::Decimal;
use serde_json::Value as J;

use crate::error::{setup, Result};

/// The key of the object a decimal is written as.
pub const DECIMAL_KEY: &str = "__decimal";

/// `value` as the chronicle holds it. A record that holds [`DECIMAL_KEY`]
/// itself, such as one a data source gave, is an error: read back, it
/// would be a decimal it never was.
pub fn to_json(value: &Value) -> Result<J> {
    Ok(match value {
        Value::Decimal(d) => {
            let mut marker = serde_json::Map::new();
            marker.insert(DECIMAL_KEY.to_string(), J::String(d.to_string()));
            J::Object(marker)
        }
        Value::Object(map) if map.contains_key(DECIMAL_KEY) => {
            return Err(setup(format!(
                "{value} holds the key '{DECIMAL_KEY}', which the chronicle keeps for a decimal"
            )))
        }
        Value::Array(items) => J::Array(items.iter().map(to_json).collect::<Result<_>>()?),
        Value::Object(map) => J::Object(
            map.iter()
                .map(|(k, v)| Ok((k.clone(), to_json(v)?)))
                .collect::<Result<_>>()?,
        ),
        other => J::from(other),
    })
}

/// The engine value the chronicle holds as `json`: the inverse of
/// [`to_json`]. An object that holds [`DECIMAL_KEY`] but is not the object
/// [`to_json`] writes is an error: the cell refuses that key in what it is
/// supplied, so only a damaged chronicle holds one.
pub fn from_json(json: &J) -> Result<Value> {
    restore(Value::from(json))
}

/// Whether `a` and `b` hold the same engine value, by the rule of the
/// engine's `EQUALS` on records ([`equal_as_written`]): a decimal by its
/// value, whatever its scale (`5.0` is `5.00`), and an integer against a
/// decimal of the same value, at any depth, an error rather than an answer.
/// The cell does not decide on its own whether `5` and `5.0` are alike.
pub fn same(a: &J, b: &J) -> Result<bool> {
    Ok(equal_as_written(&from_json(a)?, &from_json(b)?)?)
}

/// Whether `a` and `b` are the same, as [`same`], where a value may be
/// absent: two absent values are the same, one absent value is not.
pub fn same_held(a: Option<&J>, b: Option<&J>) -> Result<bool> {
    match (a, b) {
        (Some(a), Some(b)) => same(a, b),
        (a, b) => Ok(a.is_none() && b.is_none()),
    }
}

/// Turn the decimal objects in `value` back into decimals.
fn restore(value: Value) -> Result<Value> {
    Ok(match value {
        Value::Object(map) if map.contains_key(DECIMAL_KEY) => Value::Decimal(decimal(&map)?),
        Value::Object(map) => Value::Object(
            map.into_iter()
                .map(|(k, v)| Ok((k, restore(v)?)))
                .collect::<Result<_>>()?,
        ),
        Value::Array(items) => Value::Array(items.into_iter().map(restore).collect::<Result<_>>()?),
        other => other,
    })
}

/// The decimal `map` stands for: the object [`to_json`] writes for one.
fn decimal(map: &BTreeMap<String, Value>) -> Result<Decimal> {
    let digits = map
        .get(DECIMAL_KEY)
        .and_then(Value::as_str)
        .filter(|_| map.len() == 1);
    // Only the digits `to_json` writes: what parses but reads otherwise,
    // such as `1_0` or `+1`, is no decimal the chronicle wrote.
    digits
        .and_then(|d| d.parse::<Decimal>().ok().filter(|p| p.to_string() == d))
        .ok_or_else(|| setup(format!("{map:?} is no decimal the chronicle wrote")))
}

/// Whether `json`, supplied from outside the cell, holds [`DECIMAL_KEY`]
/// anywhere. The cell refuses such a value: read back, it would turn into a
/// decimal it never was.
pub fn holds_decimal_key(json: &J) -> bool {
    match json {
        J::Object(map) => map.contains_key(DECIMAL_KEY) || map.values().any(holds_decimal_key),
        J::Array(items) => items.iter().any(holds_decimal_key),
        _ => false,
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::panic)]
mod tests {
    use super::*;
    use serde_json::json;

    fn dec(s: &str) -> Value {
        Value::Decimal(s.parse().unwrap())
    }

    fn round_trip(value: &Value) -> Value {
        let text = serde_json::to_string(&to_json(value).unwrap()).unwrap();
        from_json(&serde_json::from_str(&text).unwrap()).unwrap()
    }

    #[test]
    fn a_decimal_keeps_its_digits_and_scale() {
        for s in ["5.0", "5", "0.3333333333333333333333333333", "-12.50"] {
            let value = dec(s);
            let back = round_trip(&value);
            assert!(matches!(back, Value::Decimal(_)), "{s}");
            assert_eq!(back.to_string(), s);
        }
    }

    #[test]
    fn an_integer_stays_an_integer() {
        assert!(matches!(round_trip(&Value::Int(5)), Value::Int(5)));
    }

    #[test]
    fn decimals_inside_lists_and_records_come_back() {
        let value = Value::Array(vec![Value::Object(
            [
                ("aandeel".to_string(), dec("0.5")),
                ("n".into(), Value::Int(2)),
            ]
            .into(),
        )]);
        assert_eq!(
            to_json(&value).unwrap(),
            json!([{"aandeel": {"__decimal": "0.5"}, "n": 2}])
        );
        let Value::Array(items) = round_trip(&value) else {
            panic!("not a list")
        };
        let Value::Object(map) = &items[0] else {
            panic!("not a record")
        };
        assert_eq!(map["aandeel"].to_string(), "0.5");
        assert!(matches!(map["aandeel"], Value::Decimal(_)));
        assert!(matches!(map["n"], Value::Int(2)));
    }

    #[test]
    fn a_damaged_decimal_is_an_error() {
        for json in [
            json!({"__decimal": "geen getal"}),
            json!({"__decimal": "1.5", "ander": 1}),
            json!([{"__decimal": 1.5}]),
            json!({"__decimal": "1_0"}),
            json!({"__decimal": "+1"}),
        ] {
            assert!(from_json(&json).is_err(), "{json}");
        }
    }

    #[test]
    fn a_decimal_is_the_same_whatever_its_scale() {
        let a = to_json(&dec("5.0")).unwrap();
        assert!(same(&a, &to_json(&dec("5.00")).unwrap()).unwrap());
        assert!(!same(&a, &to_json(&dec("5.01")).unwrap()).unwrap());
        assert!(!same(&a, &json!(6)).unwrap());
        assert!(same_held(Some(&a), Some(&to_json(&dec("5.00")).unwrap())).unwrap());
        assert!(same_held(None, None).unwrap());
        assert!(!same_held(Some(&a), None).unwrap());
        assert!(!same_held(None, Some(&a)).unwrap());
    }

    #[test]
    fn an_integer_against_the_same_decimal_is_an_error() {
        // Not a difference and not a match: whether 5 and 5.0 are alike is
        // a question the engine refuses to answer silently, and so does the
        // cell, bare or inside a record or a list.
        let five = to_json(&dec("5.0")).unwrap();
        for (a, b) in [
            (five.clone(), json!(5)),
            (json!({"n": five.clone()}), json!({"n": 5})),
            (json!([5]), json!([five.clone()])),
        ] {
            let err = same(&a, &b).unwrap_err();
            assert!(matches!(err, crate::error::Error::Engine(_)), "{err}");
            assert!(same_held(Some(&b), Some(&a)).is_err());
        }
    }

    #[test]
    fn other_values_are_the_json_the_engine_writes() {
        let unknown = Value::from(&json!({"__unknown": true, "missing": [
            {"law": "wet", "name": "x", "kind": "no_data"}
        ]}));
        let untranslatable = Value::Untranslatable {
            article: "1".into(),
            construct: "iets".into(),
        };
        for value in [
            Value::Null,
            Value::Bool(true),
            "tekst".into(),
            unknown,
            untranslatable,
        ] {
            assert_eq!(to_json(&value).unwrap(), J::from(&value));
            assert_eq!(round_trip(&value), value);
        }
    }

    #[test]
    fn a_record_that_holds_the_decimal_key_is_not_written() {
        let record = Value::from(&json!({"x": {"__decimal": "1.5"}}));
        assert!(to_json(&record).is_err());
        assert!(to_json(&Value::Array(vec![record])).is_err());
    }

    #[test]
    fn the_decimal_key_is_found_anywhere() {
        assert!(holds_decimal_key(&json!({"__decimal": "1"})));
        assert!(holds_decimal_key(&json!([{"a": {"__decimal": 1}}])));
        assert!(!holds_decimal_key(&json!({"a": [1, "__decimal"]})));
    }
}
