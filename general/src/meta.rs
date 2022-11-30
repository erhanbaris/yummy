use serde::{Serialize, de::Visitor, de::MapAccess, Deserialize, Deserializer};
use serde_json::Value;
use std::fmt;
use serde::de::{self};

#[derive(Debug, PartialEq, Eq, Serialize)]
pub enum Visibility {
    Anonymous = 0,
    User = 1,
    Friend = 2,
    Mod = 3,
    Admin = 4,
    System = 5
}

#[derive(Debug, Serialize, PartialEq)]
pub enum MetaType {
    Null,
    Integer(i64, Visibility),
    Float(f64, Visibility),
    String(String, Visibility),
    Bool(bool, Visibility)
}

impl<'de> Deserialize<'de> for MetaType {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_any(MetaVisitor)
    }
}

struct MetaVisitor;


impl<'de> Visitor<'de> for MetaVisitor {
    type Value = MetaType;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("meta type")
    }

    fn visit_some<D>(self, deserializer: D) -> Result<Self::Value, D::Error> where D: Deserializer<'de> {
        Ok(deserializer.deserialize_any(MetaVisitor)?)
    }

    fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E> where E: de::Error {
        Ok(MetaType::Integer(value, Visibility::Anonymous))
    }

    fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E> where E: de::Error {
        Ok(MetaType::Integer(value as i64, Visibility::Anonymous))
    }

    fn visit_f64<E>(self, value: f64) -> Result<Self::Value, E> where E: de::Error {
        Ok(MetaType::Float(value, Visibility::Anonymous))
    }

    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E> where E: de::Error {
        Ok(MetaType::String(value.to_string(), Visibility::Anonymous))
    }

    fn visit_bool<E>(self, value: bool) -> Result<Self::Value, E> where E: de::Error {
        Ok(MetaType::Bool(value, Visibility::Anonymous))
    }

    fn visit_unit<E>(self) -> Result<Self::Value, E> where E: de::Error { 
        Ok(MetaType::Null)
    }

    fn visit_map<E>(self, mut access: E) -> Result<Self::Value, E::Error> where E: MapAccess<'de> {
        let mut visibility: Option<Visibility> = None;
        let mut value: Option<serde_json::Value> = None;
        while let Some(key) = access.next_key::<&str>()? {
            match key {
                "access" => visibility = Some(match access.next_value::<usize>() {
                    Ok(0) => Visibility::Anonymous,
                    Ok(1) => Visibility::User,
                    Ok(2) => Visibility::Friend,
                    Ok(3) => Visibility::Mod,
                    Ok(4) => Visibility::Admin,
                    Ok(5) => Visibility::System,
                    _ => return Err(de::Error::custom(r#"Invalid "access" type"#))
                }),
                "value" => value = Some(access.next_value::<serde_json::Value>()?),
                _ => return Err(de::Error::custom(format!(r#""{}" is not valid"#, key)))
            }
        }

        let visibility = match visibility {
            Some(visibility) => visibility,
            None => return Err(de::Error::custom(r#""access" key is missing"#))
        };
        
        match value {
            Some(value) => match value {
                Value::Bool(bool) => Ok(MetaType::Bool(bool, visibility)),
                Value::String(string) => Ok(MetaType::String(string, visibility)),
                Value::Number(number) => {
                    if number.is_f64() {
                        Ok(MetaType::Float(number.as_f64().unwrap_or_default(), visibility))
                    } else if number.is_i64() {
                        Ok(MetaType::Integer(number.as_i64().unwrap_or_default(), visibility))
                    } else if number.is_u64() {
                        Ok(MetaType::Integer(number.as_u64().unwrap_or_default() as i64, visibility))
                    } else {
                        Err(de::Error::custom(r#"Only, number, string and bool types are valid for "value""#))
                    }
                },
                _ => Err(de::Error::custom(r#"Only, number, string and bool types are valid for "value""#))
            },
            None => Err(de::Error::custom(r#""value" key is missing"#))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json;

    #[test]
    fn basic_deserialization() {
        let s = "2";
        let deserialized: MetaType = serde_json::from_str(s).unwrap();
        assert_eq!(deserialized, MetaType::Integer(2, Visibility::Anonymous));

        let s = "\"erhan\"";
        let deserialized: MetaType = serde_json::from_str(s).unwrap();
        assert_eq!(deserialized, MetaType::String("erhan".to_string(), Visibility::Anonymous));

        let s = "true";
        let deserialized: MetaType = serde_json::from_str(s).unwrap();
        assert_eq!(deserialized, MetaType::Bool(true, Visibility::Anonymous));

        let s = "false";
        let deserialized: MetaType = serde_json::from_str(s).unwrap();
        assert_eq!(deserialized, MetaType::Bool(false, Visibility::Anonymous));

        let s = "10.5";
        let deserialized: MetaType = serde_json::from_str(s).unwrap();
        assert_eq!(deserialized, MetaType::Float(10.5, Visibility::Anonymous));

        let s = "null";
        let deserialized: MetaType = serde_json::from_str(s).unwrap();
        assert_eq!(deserialized, MetaType::Null);
    }

    #[test]
    fn dict_deserialization() {
        let s = r#"{"access": 3, "value": true}"#;
        let deserialized: MetaType = serde_json::from_str(s).unwrap();
        assert_eq!(deserialized, MetaType::Bool(true, Visibility::Mod));

        let s = r#"{"access": 3, "value": "erhan"}"#;
        let deserialized: MetaType = serde_json::from_str(s).unwrap();
        assert_eq!(deserialized, MetaType::String("erhan".to_string(), Visibility::Mod));

        let s = r#"{"access": 0, "value": true}"#;
        let deserialized: MetaType = serde_json::from_str(s).unwrap();
        assert_eq!(deserialized, serde_json::from_str("true").unwrap());
    }

    #[test]
    fn wrong_deserialization() {
        let s = r#"{"access": 3}"#;
        assert_eq!(serde_json::from_str::<MetaType>(s).err().unwrap().to_string(), r#""value" key is missing at line 1 column 13"#);

        let s = r#"{"value": true}"#;
        assert_eq!(serde_json::from_str::<MetaType>(s).err().unwrap().to_string(), r#""access" key is missing at line 1 column 15"#);

        let s = r#"{"access": 0, "value": true, "wrong": 1}"#;
        assert_eq!(serde_json::from_str::<MetaType>(s).err().unwrap().to_string(), r#""wrong" is not valid at line 1 column 36"#);

        let s = r#"{"access": "0", "value": true}"#;
        assert_eq!(serde_json::from_str::<MetaType>(s).err().unwrap().to_string(), r#"Invalid "access" type at line 1 column 14"#);

        let s = r#"{"access": 0, "value": {}}"#;
        assert_eq!(serde_json::from_str::<MetaType>(s).err().unwrap().to_string(), r#"Only, number, string and bool types are valid for "value" at line 1 column 26"#);
    }
}