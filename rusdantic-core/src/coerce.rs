//! Type coercion helpers for lax deserialization mode.
//!
//! In lax mode, Rusdantic accepts values that can be reasonably converted
//! to the target type. For example, the string `"123"` can be coerced to
//! the integer `123`, and the integer `1` can be coerced to `true`.
//!
//! This matches Pydantic V2's lax mode coercion rules.

use serde::de::{self, Deserializer, Visitor};
use std::fmt;
use std::marker::PhantomData;
use std::str::FromStr;

/// Deserialize a value with lax coercion for integer types.
/// Accepts: integer, string parseable as integer, float with no fractional part.
pub fn deserialize_coerce_int<'de, D, T>(deserializer: D) -> Result<T, D::Error>
where
    D: Deserializer<'de>,
    T: FromStr + TryFrom<i64> + TryFrom<u64>,
    <T as FromStr>::Err: fmt::Display,
    <T as TryFrom<i64>>::Error: fmt::Display,
    <T as TryFrom<u64>>::Error: fmt::Display,
{
    struct CoerceIntVisitor<T>(PhantomData<T>);

    impl<'de, T> Visitor<'de> for CoerceIntVisitor<T>
    where
        T: FromStr + TryFrom<i64> + TryFrom<u64>,
        <T as FromStr>::Err: fmt::Display,
        <T as TryFrom<i64>>::Error: fmt::Display,
        <T as TryFrom<u64>>::Error: fmt::Display,
    {
        type Value = T;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("an integer, or a string parseable as an integer")
        }

        // Accept integers directly
        fn visit_i64<E: de::Error>(self, v: i64) -> Result<T, E> {
            T::try_from(v).map_err(de::Error::custom)
        }

        fn visit_u64<E: de::Error>(self, v: u64) -> Result<T, E> {
            T::try_from(v).map_err(de::Error::custom)
        }

        // Coerce: string → integer
        fn visit_str<E: de::Error>(self, v: &str) -> Result<T, E> {
            v.trim().parse::<T>().map_err(de::Error::custom)
        }

        // Coerce: float → integer (only if no fractional part and within safe range)
        fn visit_f64<E: de::Error>(self, v: f64) -> Result<T, E> {
            if !v.is_finite() {
                return Err(de::Error::custom(format!(
                    "cannot coerce {} to integer: value is not finite",
                    v
                )));
            }
            if v.fract() != 0.0 {
                return Err(de::Error::custom(format!(
                    "cannot coerce float {} to integer: has fractional part",
                    v
                )));
            }
            // Bounds check: verify the float is within the representable range
            // of i64/u64 and that the conversion preserves precision.
            if v < 0.0 {
                if v < (i64::MIN as f64) || v > (i64::MAX as f64) {
                    return Err(de::Error::custom(format!(
                        "cannot coerce {} to integer: value out of range",
                        v
                    )));
                }
                let i = v as i64;
                // Verify round-trip precision: the f64 must exactly represent this integer
                if (i as f64) != v {
                    return Err(de::Error::custom(format!(
                        "cannot coerce {} to integer: loss of precision",
                        v
                    )));
                }
                T::try_from(i).map_err(de::Error::custom)
            } else {
                if v > (u64::MAX as f64) {
                    return Err(de::Error::custom(format!(
                        "cannot coerce {} to integer: value out of range",
                        v
                    )));
                }
                let u = v as u64;
                if (u as f64) != v {
                    return Err(de::Error::custom(format!(
                        "cannot coerce {} to integer: loss of precision",
                        v
                    )));
                }
                T::try_from(u).map_err(de::Error::custom)
            }
        }

        // Coerce: bool → integer (true=1, false=0)
        fn visit_bool<E: de::Error>(self, v: bool) -> Result<T, E> {
            if v {
                T::try_from(1i64).map_err(de::Error::custom)
            } else {
                T::try_from(0i64).map_err(de::Error::custom)
            }
        }
    }

    deserializer.deserialize_any(CoerceIntVisitor(PhantomData))
}

/// Deserialize a value with lax coercion for float types.
/// Accepts: float, integer, string parseable as float.
pub fn deserialize_coerce_float<'de, D, T>(deserializer: D) -> Result<T, D::Error>
where
    D: Deserializer<'de>,
    T: FromStr,
    <T as FromStr>::Err: fmt::Display,
{
    struct CoerceFloatVisitor<T>(PhantomData<T>);

    impl<'de, T> Visitor<'de> for CoerceFloatVisitor<T>
    where
        T: FromStr,
        <T as FromStr>::Err: fmt::Display,
    {
        type Value = T;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("a number, or a string parseable as a number")
        }

        fn visit_f64<E: de::Error>(self, v: f64) -> Result<T, E> {
            if !v.is_finite() {
                return Err(de::Error::custom(
                    "cannot coerce non-finite float (NaN/Infinity)",
                ));
            }
            // Convert f64 to string and parse to target type
            // This handles both f32 and f64 targets
            v.to_string().parse::<T>().map_err(de::Error::custom)
        }

        fn visit_i64<E: de::Error>(self, v: i64) -> Result<T, E> {
            v.to_string().parse::<T>().map_err(de::Error::custom)
        }

        fn visit_u64<E: de::Error>(self, v: u64) -> Result<T, E> {
            v.to_string().parse::<T>().map_err(de::Error::custom)
        }

        fn visit_str<E: de::Error>(self, v: &str) -> Result<T, E> {
            v.trim().parse::<T>().map_err(de::Error::custom)
        }
    }

    deserializer.deserialize_any(CoerceFloatVisitor(PhantomData))
}

/// Deserialize a value with lax coercion for bool.
/// Accepts: bool, int (0/1), string ("true"/"false"/"1"/"0"/"yes"/"no").
pub fn deserialize_coerce_bool<'de, D>(deserializer: D) -> Result<bool, D::Error>
where
    D: Deserializer<'de>,
{
    struct CoerceBoolVisitor;

    impl<'de> Visitor<'de> for CoerceBoolVisitor {
        type Value = bool;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("a boolean, 0/1, or \"true\"/\"false\"")
        }

        fn visit_bool<E: de::Error>(self, v: bool) -> Result<bool, E> {
            Ok(v)
        }

        fn visit_i64<E: de::Error>(self, v: i64) -> Result<bool, E> {
            match v {
                0 => Ok(false),
                1 => Ok(true),
                _ => Err(de::Error::custom(format!(
                    "cannot coerce integer {} to bool: expected 0 or 1",
                    v
                ))),
            }
        }

        fn visit_u64<E: de::Error>(self, v: u64) -> Result<bool, E> {
            match v {
                0 => Ok(false),
                1 => Ok(true),
                _ => Err(de::Error::custom(format!(
                    "cannot coerce integer {} to bool: expected 0 or 1",
                    v
                ))),
            }
        }

        fn visit_str<E: de::Error>(self, v: &str) -> Result<bool, E> {
            match v.trim().to_lowercase().as_str() {
                "true" | "1" | "yes" | "on" => Ok(true),
                "false" | "0" | "no" | "off" => Ok(false),
                _ => Err(de::Error::custom(format!(
                    "cannot coerce \"{}\" to bool: expected true/false/1/0/yes/no",
                    v
                ))),
            }
        }
    }

    deserializer.deserialize_any(CoerceBoolVisitor)
}

/// Deserialize a value with lax coercion for String.
/// Accepts: string, number (to_string), bool (to_string).
pub fn deserialize_coerce_string<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    struct CoerceStringVisitor;

    impl<'de> Visitor<'de> for CoerceStringVisitor {
        type Value = String;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("a string, number, or boolean")
        }

        fn visit_str<E: de::Error>(self, v: &str) -> Result<String, E> {
            Ok(v.to_string())
        }

        fn visit_string<E: de::Error>(self, v: String) -> Result<String, E> {
            Ok(v)
        }

        fn visit_i64<E: de::Error>(self, v: i64) -> Result<String, E> {
            Ok(v.to_string())
        }

        fn visit_u64<E: de::Error>(self, v: u64) -> Result<String, E> {
            Ok(v.to_string())
        }

        fn visit_f64<E: de::Error>(self, v: f64) -> Result<String, E> {
            Ok(v.to_string())
        }

        fn visit_bool<E: de::Error>(self, v: bool) -> Result<String, E> {
            Ok(v.to_string())
        }
    }

    deserializer.deserialize_any(CoerceStringVisitor)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::de::IntoDeserializer;
    use serde_json::json;

    // Helper: coerce a JSON value to i32 using our coercion function
    fn coerce_to_i32(value: serde_json::Value) -> Result<i32, serde_json::Error> {
        deserialize_coerce_int(value.into_deserializer())
    }

    #[test]
    fn test_coerce_string_to_i32() {
        assert_eq!(coerce_to_i32(json!("123")).unwrap(), 123);
    }

    #[test]
    fn test_coerce_string_to_i32_negative() {
        assert_eq!(coerce_to_i32(json!("-42")).unwrap(), -42);
    }

    #[test]
    fn test_coerce_string_to_i32_invalid() {
        assert!(coerce_to_i32(json!("abc")).is_err());
    }

    #[test]
    fn test_coerce_float_to_i32_no_fraction() {
        assert_eq!(coerce_to_i32(json!(42.0)).unwrap(), 42);
    }

    #[test]
    fn test_coerce_float_to_i32_with_fraction_fails() {
        assert!(coerce_to_i32(json!(42.5)).is_err());
    }

    #[test]
    fn test_coerce_bool_to_i32() {
        assert_eq!(coerce_to_i32(json!(true)).unwrap(), 1);
        assert_eq!(coerce_to_i32(json!(false)).unwrap(), 0);
    }

    fn coerce_to_bool(value: serde_json::Value) -> Result<bool, serde_json::Error> {
        deserialize_coerce_bool(value.into_deserializer())
    }

    fn coerce_to_string(value: serde_json::Value) -> Result<String, serde_json::Error> {
        deserialize_coerce_string(value.into_deserializer())
    }

    fn coerce_to_f64(value: serde_json::Value) -> Result<f64, serde_json::Error> {
        deserialize_coerce_float(value.into_deserializer())
    }

    fn coerce_to_u8(value: serde_json::Value) -> Result<u8, serde_json::Error> {
        deserialize_coerce_int(value.into_deserializer())
    }

    #[test]
    fn test_coerce_string_to_bool() {
        assert!(coerce_to_bool(json!("true")).unwrap());
        assert!(!coerce_to_bool(json!("false")).unwrap());
        assert!(coerce_to_bool(json!("yes")).unwrap());
        assert!(!coerce_to_bool(json!("0")).unwrap());
    }

    #[test]
    fn test_coerce_int_to_bool() {
        assert!(coerce_to_bool(json!(1)).unwrap());
        assert!(!coerce_to_bool(json!(0)).unwrap());
    }

    #[test]
    fn test_coerce_int_to_bool_invalid() {
        assert!(coerce_to_bool(json!(2)).is_err());
    }

    #[test]
    fn test_coerce_number_to_string() {
        assert_eq!(coerce_to_string(json!(42)).unwrap(), "42");
        assert_eq!(coerce_to_string(json!(3.15)).unwrap(), "3.15");
        assert_eq!(coerce_to_string(json!(true)).unwrap(), "true");
    }

    #[test]
    fn test_coerce_string_to_f64() {
        assert!((coerce_to_f64(json!("3.15")).unwrap() - 3.15).abs() < f64::EPSILON);
        assert!((coerce_to_f64(json!(42)).unwrap() - 42.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_coerce_u8_range() {
        assert_eq!(coerce_to_u8(json!("255")).unwrap(), 255);
        assert!(coerce_to_u8(json!("256")).is_err()); // overflow
    }
}
