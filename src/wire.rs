//! Version 3 native save wire: bounded, deterministic DEFLATE and hexadecimal.
//! Snapshot still owns framing, checksums, migrations and transactional restoration.
use crate::city::City;
use serde::{Deserialize, Serialize};
use std::cell::Cell;
thread_local! {
    // Hash the complete plain state without recompressing the city at every
    // replay tick. The scoped flag is restored even if serialization panics.
    static HASHING: Cell<bool> = const { Cell::new(false) };
}
pub(crate) fn state_hash(state: &crate::State) -> u64 {
    struct Guard(bool);
    impl Drop for Guard {
        fn drop(&mut self) {
            HASHING.set(self.0);
        }
    }
    let _guard = Guard(HASHING.replace(true));
    vesper3d::two_d::hash_json(state)
}
const MAX_DECODED: usize = 64 * 1024 * 1024;
const MAX_COMPRESSED: usize = 16 * 1024 * 1024;
#[derive(Serialize)]
struct Archive<'a> {
    codec: &'static str,
    data: &'a str,
}
pub fn serialize<S: serde::Serializer>(city: &City, s: S) -> Result<S::Ok, S::Error> {
    if HASHING.get() {
        return city.serialize(s);
    }
    let raw = serde_json::to_vec(city).map_err(serde::ser::Error::custom)?;
    if raw.len() > MAX_DECODED {
        return Err(serde::ser::Error::custom(
            "City exceeds the decoded save limit.",
        ));
    }
    let zipped = miniz_oxide::deflate::compress_to_vec_zlib(&raw, 1);
    if zipped.len() > MAX_COMPRESSED {
        return Err(serde::ser::Error::custom(
            "City exceeds the compressed save limit.",
        ));
    }
    let mut data = String::with_capacity(zipped.len() * 2);
    const HEX: &[u8] = b"0123456789abcdef";
    for b in zipped {
        data.push(HEX[(b >> 4) as usize] as char);
        data.push(HEX[(b & 15) as usize] as char);
    }
    Archive {
        codec: "bluecity-deflate-hex-1",
        data: &data,
    }
    .serialize(s)
}
pub fn deserialize<'de, D: serde::Deserializer<'de>>(d: D) -> Result<City, D::Error> {
    let value = serde_json::Value::deserialize(d)?;
    if let Some(codec) = value.get("codec") {
        if codec != "bluecity-deflate-hex-1" {
            return Err(serde::de::Error::custom("Unknown city archive codec."));
        }
        let data = value
            .get("data")
            .and_then(serde_json::Value::as_str)
            .ok_or_else(|| serde::de::Error::custom("Missing city archive data."))?;
        if data.len() > MAX_COMPRESSED * 2 || !data.len().is_multiple_of(2) {
            return Err(serde::de::Error::custom("Invalid city archive length."));
        }
        fn digit(b: u8) -> Option<u8> {
            match b {
                b'0'..=b'9' => Some(b - b'0'),
                b'a'..=b'f' => Some(b - b'a' + 10),
                _ => None,
            }
        }
        let bytes: Result<Vec<_>, D::Error> = data
            .as_bytes()
            .as_chunks::<2>()
            .0
            .iter()
            .map(|p| {
                digit(p[0])
                    .zip(digit(p[1]))
                    .map(|(a, b)| a * 16 + b)
                    .ok_or_else(|| serde::de::Error::custom("Invalid city archive encoding."))
            })
            .collect();
        let decoded = miniz_oxide::inflate::decompress_to_vec_zlib_with_limit(&bytes?, MAX_DECODED)
            .map_err(|e| serde::de::Error::custom(format!("Invalid city archive: {e:?}")))?;
        serde_json::from_slice(&decoded).map_err(serde::de::Error::custom)
    } else {
        serde_json::from_value(value).map_err(serde::de::Error::custom)
    }
}
