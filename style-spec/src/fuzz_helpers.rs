//! Bounded [`arbitrary::Unstructured`] helpers for generated `#[arbitrary(with = ...)]` fields.
//!
//! Depth and collection sizes are capped so expression trees and JSON literals stay tractable.

use arbitrary::{Arbitrary, Unstructured};
use color::DynamicColor;
use geojson::{GeoJson, Geometry, Value as GeoJsonValue};
use serde_json::{Map, Number, Value};

const MAX_JSON_DEPTH: u8 = 5;
const MAX_JSON_ARRAY_LEN: usize = 8;
const MAX_JSON_OBJECT_LEN: usize = 6;
const MAX_JSON_STRING_LEN: usize = 48;

/// Valid CSS / MapLibre color strings (serde round-trips reliably).
const COLOR_SAMPLES: &[&str] = &[
    "#000",
    "#000000",
    "#ffffff",
    "#FfFfFf",
    "white",
    "black",
    "red",
    "rgb(0,128,255)",
    "rgba(10,20,30,0.5)",
    "hsl(200,50%,50%)",
    "transparent",
];

pub fn arbitrary_json_number(u: &mut Unstructured<'_>) -> arbitrary::Result<Number> {
    let tag: u8 = u.arbitrary()?;
    match tag % 5 {
        0 => Ok(Number::from(i64::arbitrary(u)?)),
        1 => Ok(Number::from(u64::arbitrary(u)?)),
        2 => Ok(Number::from(i32::arbitrary(u)? as i64)),
        3 => {
            let x: f64 = f64::arbitrary(u)?;
            Number::from_f64(x).ok_or(arbitrary::Error::IncorrectFormat)
        }
        _ => Ok(Number::from(0i32)),
    }
}

fn arbitrary_short_string(u: &mut Unstructured<'_>) -> arbitrary::Result<String> {
    let len = u.int_in_range(0usize..=MAX_JSON_STRING_LEN)?;
    let mut s = String::with_capacity(len);
    for _ in 0..len {
        let c: u8 = u.int_in_range(b' '..=b'~')?;
        s.push(c as char);
    }
    Ok(s)
}

/// JSON [`Value`] with depth and container size limits.
pub fn arbitrary_json_value(u: &mut Unstructured<'_>) -> arbitrary::Result<Value> {
    arbitrary_json_value_inner(u, 0)
}

pub fn arbitrary_json_map(u: &mut Unstructured<'_>) -> arbitrary::Result<Map<String, Value>> {
    let n = u.int_in_range(0..=MAX_JSON_OBJECT_LEN)?;
    let mut m = Map::new();
    for i in 0..n {
        let mut key = arbitrary_short_string(u)?;
        if key.is_empty() {
            key = format!("k{i}");
        }
        m.insert(key, arbitrary_json_value(u)?);
    }
    Ok(m)
}

pub fn arbitrary_option_json_map(
    u: &mut Unstructured<'_>,
) -> arbitrary::Result<Option<Map<String, Value>>> {
    let present: bool = u.arbitrary()?;
    if present {
        arbitrary_json_map(u).map(Some)
    } else {
        Ok(None)
    }
}

fn arbitrary_json_value_inner(u: &mut Unstructured<'_>, depth: u8) -> arbitrary::Result<Value> {
    if depth >= MAX_JSON_DEPTH {
        return leaf_json(u);
    }

    let arm: u8 = u.arbitrary()?;
    // Avoid `Value::Null`: serde_json conflates `Some(Value::Null)` with `None`
    // inside `Option<T>`, breaking round-trip equality assertions.
    match arm % 6 {
        0 => Ok(Value::Bool(bool::arbitrary(u)?)),
        1 => arbitrary_json_number(u).map(Value::Number),
        2 => arbitrary_short_string(u).map(Value::String),
        3 => {
            let n = u.int_in_range(0..=MAX_JSON_ARRAY_LEN)?;
            let mut v = Vec::with_capacity(n);
            for _ in 0..n {
                v.push(arbitrary_json_value_inner(u, depth + 1)?);
            }
            Ok(Value::Array(v))
        }
        _ => {
            let n = u.int_in_range(0..=MAX_JSON_OBJECT_LEN)?;
            let mut m = serde_json::Map::new();
            for i in 0..n {
                let mut key = arbitrary_short_string(u)?;
                if key.is_empty() {
                    key = format!("k{i}");
                }
                m.insert(key, arbitrary_json_value_inner(u, depth + 1)?);
            }
            Ok(Value::Object(m))
        }
    }
}

fn leaf_json(u: &mut Unstructured<'_>) -> arbitrary::Result<Value> {
    let arm: u8 = u.arbitrary()?;
    match arm % 3 {
        0 => Ok(Value::Bool(bool::arbitrary(u)?)),
        1 => arbitrary_json_number(u).map(Value::Number),
        _ => arbitrary_short_string(u).map(Value::String),
    }
}

pub fn arbitrary_option_json_value(u: &mut Unstructured<'_>) -> arbitrary::Result<Option<Value>> {
    let present: bool = u.arbitrary()?;
    if present {
        arbitrary_json_value(u).map(Some)
    } else {
        Ok(None)
    }
}

pub fn arbitrary_url(u: &mut Unstructured<'_>) -> arbitrary::Result<url::Url> {
    for _ in 0..8 {
        let s = arbitrary_short_string(u)?;
        let candidate = if s.is_empty() {
            "https://example.com/".to_string()
        } else {
            format!("https://example.com/{s}")
        };
        if let Ok(parsed) = url::Url::parse(&candidate) {
            return Ok(parsed);
        }
    }
    url::Url::parse("https://example.com/").map_err(|_| arbitrary::Error::IncorrectFormat)
}

pub fn arbitrary_vec_json_value(u: &mut Unstructured<'_>) -> arbitrary::Result<Vec<Value>> {
    let n = u.int_in_range(0..=MAX_JSON_ARRAY_LEN)?;
    let mut v = Vec::with_capacity(n);
    for _ in 0..n {
        v.push(arbitrary_json_value(u)?);
    }
    Ok(v)
}

#[expect(
    dead_code,
    reason = "referenced in generator test snapshots but not in current generated spec"
)]
pub fn arbitrary_vec_json_number(u: &mut Unstructured<'_>) -> arbitrary::Result<Vec<Number>> {
    let n = u.int_in_range(0..=MAX_JSON_ARRAY_LEN)?;
    let mut v = Vec::with_capacity(n);
    for _ in 0..n {
        v.push(arbitrary_json_number(u)?);
    }
    Ok(v)
}

fn arbitrary_box_json_numbers<const N: usize>(
    u: &mut Unstructured<'_>,
) -> arbitrary::Result<Box<[Number; N]>> {
    let mut v = Vec::with_capacity(N);
    for _ in 0..N {
        v.push(arbitrary_json_number(u)?);
    }
    let a: [Number; N] = v
        .try_into()
        .map_err(|_| arbitrary::Error::IncorrectFormat)?;
    Ok(Box::new(a))
}

#[expect(
    dead_code,
    reason = "referenced in generator test snapshots but not in current generated spec"
)]
pub fn arbitrary_box_1_json_number(
    u: &mut Unstructured<'_>,
) -> arbitrary::Result<Box<[Number; 1]>> {
    arbitrary_box_json_numbers::<1>(u)
}

pub fn arbitrary_box_2_json_number(
    u: &mut Unstructured<'_>,
) -> arbitrary::Result<Box<[Number; 2]>> {
    arbitrary_box_json_numbers::<2>(u)
}

#[expect(
    dead_code,
    reason = "referenced in generator test snapshots but not in current generated spec"
)]
pub fn arbitrary_box_3_json_number(
    u: &mut Unstructured<'_>,
) -> arbitrary::Result<Box<[Number; 3]>> {
    arbitrary_box_json_numbers::<3>(u)
}

pub fn arbitrary_box_4_json_number(
    u: &mut Unstructured<'_>,
) -> arbitrary::Result<Box<[Number; 4]>> {
    arbitrary_box_json_numbers::<4>(u)
}

#[expect(
    dead_code,
    reason = "referenced in generator test snapshots but not in current generated spec"
)]
pub fn arbitrary_vec_dynamic_color(
    u: &mut Unstructured<'_>,
) -> arbitrary::Result<Vec<DynamicColor>> {
    let n = u.int_in_range(0..=MAX_JSON_ARRAY_LEN)?;
    let mut v = Vec::with_capacity(n);
    for _ in 0..n {
        v.push(arbitrary_dynamic_color(u)?);
    }
    Ok(v)
}

pub fn arbitrary_dynamic_color(u: &mut Unstructured<'_>) -> arbitrary::Result<DynamicColor> {
    let pick: u8 = u.arbitrary()?;
    if pick < 200 {
        let idx = (pick as usize) % COLOR_SAMPLES.len();
        return color::parse_color(COLOR_SAMPLES[idx])
            .map_err(|_| arbitrary::Error::IncorrectFormat);
    }
    let s = arbitrary_short_string(u)?;
    color::parse_color(&s)
        .or_else(|_| color::parse_color("#808080"))
        .map_err(|_| arbitrary::Error::IncorrectFormat)
}

pub fn arbitrary_geojson(u: &mut Unstructured<'_>) -> arbitrary::Result<GeoJson> {
    // f64::arbitrary can produce NaN/infinity
    let lon = i16::arbitrary(u)? as f64;
    let lat = i16::arbitrary(u)? as f64;
    let geom = Geometry::from(GeoJsonValue::Point(vec![lon, lat]));
    Ok(GeoJson::Geometry(geom))
}
