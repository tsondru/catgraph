use crate::error::PersistError;
use std::fmt::Debug;

/// Trait for types that can be serialized to/from JSON for SurrealDB storage.
///
/// catgraph's Lambda type parameter doesn't require serde traits, so this trait
/// bridges the gap by providing JSON serialization for the subset of types
/// actually used as labels.
pub trait Persistable: Sized + Eq + Clone + Debug {
    fn to_json_value(&self) -> serde_json::Value;
    fn from_json_value(v: &serde_json::Value) -> Result<Self, PersistError>;
    fn type_name() -> &'static str;
}

impl Persistable for char {
    fn to_json_value(&self) -> serde_json::Value {
        serde_json::Value::String(self.to_string())
    }

    fn from_json_value(v: &serde_json::Value) -> Result<Self, PersistError> {
        let s = v.as_str().ok_or_else(|| PersistError::TypeMismatch {
            expected: "string".into(),
            got: format!("{v}"),
        })?;
        let mut chars = s.chars();
        let c = chars.next().ok_or_else(|| PersistError::InvalidData("empty string for char".into()))?;
        if chars.next().is_some() {
            return Err(PersistError::InvalidData(format!("multi-char string '{s}' for char")));
        }
        Ok(c)
    }

    fn type_name() -> &'static str {
        "char"
    }
}

impl Persistable for () {
    fn to_json_value(&self) -> serde_json::Value {
        serde_json::Value::Null
    }

    fn from_json_value(v: &serde_json::Value) -> Result<Self, PersistError> {
        if v.is_null() {
            Ok(())
        } else {
            Err(PersistError::TypeMismatch {
                expected: "null".into(),
                got: format!("{v}"),
            })
        }
    }

    fn type_name() -> &'static str {
        "unit"
    }
}

impl Persistable for u32 {
    fn to_json_value(&self) -> serde_json::Value {
        serde_json::Value::Number((*self).into())
    }

    fn from_json_value(v: &serde_json::Value) -> Result<Self, PersistError> {
        v.as_u64()
            .and_then(|n| u32::try_from(n).ok())
            .ok_or_else(|| PersistError::TypeMismatch {
                expected: "u32".into(),
                got: format!("{v}"),
            })
    }

    fn type_name() -> &'static str {
        "u32"
    }
}

impl Persistable for String {
    fn to_json_value(&self) -> serde_json::Value {
        serde_json::Value::String(self.clone())
    }

    fn from_json_value(v: &serde_json::Value) -> Result<Self, PersistError> {
        v.as_str()
            .map(String::from)
            .ok_or_else(|| PersistError::TypeMismatch {
                expected: "string".into(),
                got: format!("{v}"),
            })
    }

    fn type_name() -> &'static str {
        "string"
    }
}

impl Persistable for i32 {
    fn to_json_value(&self) -> serde_json::Value {
        serde_json::Value::Number((*self).into())
    }

    fn from_json_value(v: &serde_json::Value) -> Result<Self, PersistError> {
        v.as_i64()
            .and_then(|n| i32::try_from(n).ok())
            .ok_or_else(|| PersistError::TypeMismatch {
                expected: "i32".into(),
                got: format!("{v}"),
            })
    }

    fn type_name() -> &'static str {
        "i32"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn char_roundtrip() {
        let c = 'x';
        let v = c.to_json_value();
        assert_eq!(char::from_json_value(&v).unwrap(), c);
    }

    #[test]
    fn unit_roundtrip() {
        let u = ();
        let v = u.to_json_value();
        assert_eq!(<()>::from_json_value(&v).unwrap(), u);
    }

    #[test]
    fn u32_roundtrip() {
        let n: u32 = 42;
        let v = n.to_json_value();
        assert_eq!(u32::from_json_value(&v).unwrap(), n);
    }

    #[test]
    fn i32_roundtrip() {
        let n: i32 = -7;
        let v = n.to_json_value();
        assert_eq!(i32::from_json_value(&v).unwrap(), n);
    }

    #[test]
    fn char_invalid() {
        let v = serde_json::Value::Number(42.into());
        assert!(char::from_json_value(&v).is_err());
    }

    #[test]
    fn char_multichar() {
        let v = serde_json::Value::String("ab".into());
        assert!(char::from_json_value(&v).is_err());
    }

    #[test]
    fn unit_invalid() {
        let v = serde_json::Value::String("x".into());
        assert!(<()>::from_json_value(&v).is_err());
    }

    #[test]
    fn string_roundtrip() {
        let s = String::from("hello world");
        let v = s.to_json_value();
        assert_eq!(String::from_json_value(&v).unwrap(), s);
    }

    #[test]
    fn string_empty() {
        let s = String::new();
        let v = s.to_json_value();
        assert_eq!(String::from_json_value(&v).unwrap(), s);
    }

    #[test]
    fn string_invalid() {
        let v = serde_json::Value::Number(42.into());
        assert!(String::from_json_value(&v).is_err());
    }
}
