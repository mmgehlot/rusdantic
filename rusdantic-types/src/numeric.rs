//! Constrained numeric types.
//!
//! These newtypes enforce numeric constraints at construction time and
//! during deserialization. They implement `Deref` for transparent access
//! to the inner value.

use serde::{Deserialize, Serialize};
use std::fmt;
use std::ops::Deref;

/// Macro to generate a constrained numeric newtype with validation.
macro_rules! constrained_int {
    (
        $(#[$meta:meta])*
        $name:ident, $check:expr, $err_msg:expr
    ) => {
        $(#[$meta])*
        #[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub struct $name<T: PartialOrd + Default + Copy + fmt::Display>(T);

        impl<T: PartialOrd + Default + Copy + fmt::Display> $name<T> {
            /// Create a new constrained value, returning an error if the constraint is violated.
            pub fn new(value: T) -> Result<Self, String> {
                let check: fn(&T) -> bool = $check;
                if check(&value) {
                    Ok(Self(value))
                } else {
                    Err(format!("{}: got {}", $err_msg, value))
                }
            }

            /// Get the inner value.
            pub fn into_inner(self) -> T {
                self.0
            }
        }

        impl<T: PartialOrd + Default + Copy + fmt::Display> Deref for $name<T> {
            type Target = T;
            fn deref(&self) -> &T {
                &self.0
            }
        }

        impl<T: PartialOrd + Default + Copy + fmt::Display + fmt::Debug> fmt::Debug for $name<T> {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "{}({:?})", stringify!($name), self.0)
            }
        }

        impl<T: PartialOrd + Default + Copy + fmt::Display> fmt::Display for $name<T> {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "{}", self.0)
            }
        }

        impl<T: PartialOrd + Default + Copy + fmt::Display + Serialize> Serialize for $name<T> {
            fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
                self.0.serialize(serializer)
            }
        }

        impl<'de, T> Deserialize<'de> for $name<T>
        where
            T: PartialOrd + Default + Copy + fmt::Display + Deserialize<'de>,
        {
            fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
                let value = T::deserialize(deserializer)?;
                Self::new(value).map_err(serde::de::Error::custom)
            }
        }
    };
}

constrained_int!(
    /// A positive integer (value > 0).
    ///
    /// # Example
    /// ```
    /// use rusdantic_types::PositiveInt;
    /// let p = PositiveInt::<i32>::new(42).unwrap();
    /// assert_eq!(*p, 42);
    /// assert!(PositiveInt::<i32>::new(0).is_err());
    /// assert!(PositiveInt::<i32>::new(-1).is_err());
    /// ```
    PositiveInt,
    |v: &T| *v > T::default(),
    "value must be positive (> 0)"
);

constrained_int!(
    /// A negative integer (value < 0).
    NegativeInt,
    |v: &T| *v < T::default(),
    "value must be negative (< 0)"
);

constrained_int!(
    /// A non-negative integer (value >= 0).
    NonNegativeInt,
    |v: &T| *v >= T::default(),
    "value must be non-negative (>= 0)"
);

constrained_int!(
    /// A non-positive integer (value <= 0).
    NonPositiveInt,
    |v: &T| *v <= T::default(),
    "value must be non-positive (<= 0)"
);

/// A finite float (no NaN or infinity).
///
/// # Example
/// ```
/// use rusdantic_types::FiniteFloat;
/// let f = FiniteFloat::<f64>::new(3.15).unwrap();
/// assert_eq!(*f, 3.15);
/// assert!(FiniteFloat::<f64>::new(f64::NAN).is_err());
/// assert!(FiniteFloat::<f64>::new(f64::INFINITY).is_err());
/// ```
#[derive(Clone, Copy, PartialEq, PartialOrd)]
pub struct FiniteFloat<T: PartialOrd + Copy>(T);

impl FiniteFloat<f64> {
    /// Create a new finite f64, rejecting NaN and infinity.
    pub fn new(value: f64) -> Result<Self, String> {
        if value.is_finite() {
            Ok(Self(value))
        } else {
            Err(format!("value must be finite, got {}", value))
        }
    }

    /// Get the inner value.
    pub fn into_inner(self) -> f64 {
        self.0
    }
}

impl FiniteFloat<f32> {
    /// Create a new finite f32, rejecting NaN and infinity.
    pub fn new(value: f32) -> Result<Self, String> {
        if value.is_finite() {
            Ok(Self(value))
        } else {
            Err(format!("value must be finite, got {}", value))
        }
    }

    /// Get the inner value.
    pub fn into_inner(self) -> f32 {
        self.0
    }
}

impl<T: PartialOrd + Copy> Deref for FiniteFloat<T> {
    type Target = T;
    fn deref(&self) -> &T {
        &self.0
    }
}

impl<T: PartialOrd + Copy + fmt::Debug> fmt::Debug for FiniteFloat<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "FiniteFloat({:?})", self.0)
    }
}

impl<T: PartialOrd + Copy + fmt::Display> fmt::Display for FiniteFloat<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl<T: PartialOrd + Copy + Serialize> Serialize for FiniteFloat<T> {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.0.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for FiniteFloat<f64> {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = f64::deserialize(deserializer)?;
        Self::new(value).map_err(serde::de::Error::custom)
    }
}

impl<'de> Deserialize<'de> for FiniteFloat<f32> {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = f32::deserialize(deserializer)?;
        Self::new(value).map_err(serde::de::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_positive_int_valid() {
        assert!(PositiveInt::<i32>::new(1).is_ok());
        assert!(PositiveInt::<i32>::new(100).is_ok());
    }

    #[test]
    fn test_positive_int_invalid() {
        assert!(PositiveInt::<i32>::new(0).is_err());
        assert!(PositiveInt::<i32>::new(-1).is_err());
    }

    #[test]
    fn test_negative_int() {
        assert!(NegativeInt::<i32>::new(-1).is_ok());
        assert!(NegativeInt::<i32>::new(0).is_err());
        assert!(NegativeInt::<i32>::new(1).is_err());
    }

    #[test]
    fn test_non_negative_int() {
        assert!(NonNegativeInt::<i32>::new(0).is_ok());
        assert!(NonNegativeInt::<i32>::new(1).is_ok());
        assert!(NonNegativeInt::<i32>::new(-1).is_err());
    }

    #[test]
    fn test_non_positive_int() {
        assert!(NonPositiveInt::<i32>::new(0).is_ok());
        assert!(NonPositiveInt::<i32>::new(-1).is_ok());
        assert!(NonPositiveInt::<i32>::new(1).is_err());
    }

    #[test]
    fn test_positive_int_deref() {
        let p = PositiveInt::<i32>::new(42).unwrap();
        assert_eq!(*p, 42);
        assert_eq!(p.into_inner(), 42);
    }

    #[test]
    fn test_positive_int_serialize() {
        let p = PositiveInt::<i32>::new(42).unwrap();
        let json = serde_json::to_value(p).unwrap();
        assert_eq!(json, serde_json::json!(42));
    }

    #[test]
    fn test_positive_int_deserialize_valid() {
        let p: PositiveInt<i32> = serde_json::from_value(serde_json::json!(42)).unwrap();
        assert_eq!(*p, 42);
    }

    #[test]
    fn test_positive_int_deserialize_invalid() {
        let result: Result<PositiveInt<i32>, _> = serde_json::from_value(serde_json::json!(0));
        assert!(result.is_err());
    }

    #[test]
    fn test_finite_float_valid() {
        assert!(FiniteFloat::<f64>::new(3.15).is_ok());
        assert!(FiniteFloat::<f64>::new(0.0).is_ok());
        assert!(FiniteFloat::<f64>::new(-1.0).is_ok());
    }

    #[test]
    fn test_finite_float_nan() {
        assert!(FiniteFloat::<f64>::new(f64::NAN).is_err());
    }

    #[test]
    fn test_finite_float_infinity() {
        assert!(FiniteFloat::<f64>::new(f64::INFINITY).is_err());
        assert!(FiniteFloat::<f64>::new(f64::NEG_INFINITY).is_err());
    }

    #[test]
    fn test_positive_int_u8() {
        assert!(PositiveInt::<u8>::new(1).is_ok());
        assert!(PositiveInt::<u8>::new(0).is_err());
    }

    #[test]
    fn test_display() {
        let p = PositiveInt::<i32>::new(42).unwrap();
        assert_eq!(format!("{}", p), "42");
    }
}
