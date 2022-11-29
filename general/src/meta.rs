use serde::ser::SerializeMap;
use serde::{Serialize, Serializer, de::Visitor, de::MapAccess, Deserialize, Deserializer};
use std::collections::HashMap;
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
        formatter.write_str("an integer between -2^31 and 2^31")
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

	fn visit_map<E>(self, mut access: E) -> Result<Self::Value, E::Error> where E: MapAccess<'de> {
        let mut result = MetaType::Bool(false, Visibility::Admin);
        /*while let Some((key, value)) = access.next_entry()? {
            
        }*/
        Ok(MetaType::Bool(false, Visibility::Admin))
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use serde_json;

	#[test]
	fn id_deserialization() {
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
	}
}