use std::collections::HashMap;

use eyre::{eyre, Result};
use serde::Deserialize;

pub fn validate_label_key(key: &str) -> Result<()> {
    let validations = [
        (key.is_ascii(), "must be ASCII"),
        (key.len() <= 63, "must be 63 characters or less"),
        (
            key.chars()
                .next()
                .map_or(false, |c| c.is_ascii_alphanumeric()),
            "must begin with an alphanumeric character",
        ),
        (
            key.chars()
                .last()
                .map_or(false, |c| c.is_ascii_alphanumeric()),
            "must end with an alphanumeric character",
        ),
        (
            key.chars()
                .all(|c| c.is_ascii_alphanumeric() || ['-', '_', '.'].contains(&c)),
            "must contain only alphanumeric characters, dashes, underscores, and dots",
        ),
    ];

    let mut err_msgs = vec![];
    for (valid, msg) in validations.iter() {
        if !valid {
            err_msgs.push(*msg);
        }
    }

    if err_msgs.is_empty() {
        Ok(())
    } else {
        Err(eyre!(
            "label key invalid - {} - found key : \"{}\"",
            err_msgs.join(", "),
            key
        ))
    }
}

pub fn parse_validate_label_raw(raw: &str) -> Result<(String, String)> {
    let mut split = raw.split(':');

    let mut err_msgs = vec![];
    let validations = [
        (split.clone().count() > 1, "must have a ':' character"),
        (
            split.clone().count() < 3,
            "must have only one ':' character",
        ),
    ];

    for (valid, msg) in validations.iter() {
        if !valid {
            err_msgs.push(*msg);
        }
    }

    if err_msgs.is_empty() {
        let key = split.next().unwrap_or("").to_string();
        let value = split.next().unwrap_or("").to_string();
        Ok((key, value))
    } else {
        Err(eyre!(
            "query invalid - {} - raw : \"{}\"",
            err_msgs.join(", "),
            raw
        ))
    }
}

pub fn validate_label_value(value: &str) -> Result<()> {
    // empty string is ok
    if value.is_empty() {
        return Ok(());
    }
    let validations = [
        (value.is_ascii(), "must be ASCII"),
        (value.len() <= 63, "must be 63 characters or less"),
        (
            value
                .chars()
                .next()
                .map_or(false, |c| c.is_ascii_alphanumeric()),
            "must begin with an alphanumeric character",
        ),
        (
            value
                .chars()
                .last()
                .map_or(false, |c| c.is_ascii_alphanumeric()),
            "must end with an alphanumeric character",
        ),
        (
            value
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || ['-', '_', '.'].contains(&c)),
            "must contain only alphanumeric characters, dashes, underscores, and dots",
        ),
    ];

    let mut err_msgs = vec![];
    for (valid, msg) in validations.iter() {
        if !valid {
            err_msgs.push(*msg);
        }
    }

    if err_msgs.is_empty() {
        Ok(())
    } else {
        Err(eyre!(
            "label value invalid - {} - found value : \"{}\"",
            err_msgs.join(", "),
            value
        ))
    }
}

pub fn deserialize_labels_eq_filter<'de, D>(
    deserializer: D,
) -> Result<Option<HashMap<String, serde_json::Value>>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let err_formatter = |msg: String, e: String| -> D::Error {
        serde::de::Error::custom(format!("invalid labels_eq filter - {}: {}", msg, e))
    };

    // labels_eq is in the form labels_eq=key1:value1,key2:value2
    // split on comma
    let labels: String = match Option::deserialize(deserializer)? {
        Some(labels) => labels,
        None => return Ok(None),
    };
    let labels: Vec<&str> = labels.split(',').collect();

    // if there's one label and it's an empty string (i.e. labels_eq=)
    // this is invalid - if a user wants to match on no labels,
    // they should remove the labels_eq filter entirely
    if labels.len() == 1 && labels[0].is_empty() {
        return Err(err_formatter(
            "query invalid".to_string(),
            "must have at least one label - if you want to match on no labels, remove the \
             labels_eq filter entirely"
                .to_string(),
        ));
    }

    // if there are labels, then parse them
    let mut labels_eq = HashMap::new();
    for label in labels {
        let (key, value) = parse_validate_label_raw(label)
            .map_err(|e| err_formatter("query invalid".to_string(), e.to_string()))?;

        // if the key already exists, then it's a duplicate
        if labels_eq.contains_key(&key) {
            return Err(err_formatter(
                "query has duplicate key".to_string(),
                label.to_string(),
            ));
        }
        validate_label_key(key.as_str())
            .map_err(|e| err_formatter("key invalid".to_string(), e.to_string()))?;
        validate_label_value(value.as_str())
            .map_err(|e| err_formatter("value invalid".to_string(), e.to_string()))?;

        let value = serde_json::from_str(&value).unwrap_or(serde_json::json!(value));
        labels_eq.insert(key, value);
    }

    for (key, _) in labels_eq.clone() {
        // if the first part is empty, then it's invalid
        if key.is_empty() {
            return Err(serde::de::Error::custom(
                "invalid labels_eq filter - must be in the form 'key:value' or 'key:' or ''"
                    .to_string(),
            ));
        }
    }

    Ok(Some(labels_eq))
}
