use serde::{Serialize, de::Visitor, de::MapAccess, Deserialize, Deserializer, Serializer};
use serde_json::Value;
use std::{fmt::{self, Debug}, marker::PhantomData};
use serde::de::{self};

#[derive(Debug, PartialEq, Eq, Clone, Default, Serialize)]
pub enum UserMetaAccess {
    #[default]
    Anonymous = 0,
    User = 1,
    Friend = 2,
    Me = 3,
    Mod = 4,
    Admin = 5,
    System = 6
}

impl From<UserMetaAccess> for i32 {
    fn from(access: UserMetaAccess) -> Self {
        match access {
            UserMetaAccess::Anonymous => 0,
            UserMetaAccess::User => 1,
            UserMetaAccess::Friend => 2,
            UserMetaAccess::Me => 3,
            UserMetaAccess::Mod => 4,
            UserMetaAccess::Admin => 5,
            UserMetaAccess::System => 6,
        }
    }
}

impl From<i32> for UserMetaAccess {
    fn from(access: i32) -> Self {
        match access {
            0 => UserMetaAccess::Anonymous,
            1 => UserMetaAccess::User,
            2 => UserMetaAccess::Friend,
            3 => UserMetaAccess::Me,
            4 => UserMetaAccess::Mod,
            5 => UserMetaAccess::Admin,
            6 => UserMetaAccess::System,
            _ => UserMetaAccess::default()
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Default, Serialize)]
pub enum RoomMetaAccess {
    #[default]
    Anonymous = 0,
    User = 1,
    Moderator = 2,
    Owner = 3,
    Admin = 4,
    System = 5
}

impl From<RoomMetaAccess> for i32 {
    fn from(access: RoomMetaAccess) -> Self {
        match access {
            RoomMetaAccess::Anonymous => 0,
            RoomMetaAccess::User => 1,
            RoomMetaAccess::Moderator => 2,
            RoomMetaAccess::Owner => 3,
            RoomMetaAccess::Admin => 4,
            RoomMetaAccess::System => 5,
        }
    }
}

impl From<i32> for RoomMetaAccess {
    fn from(access: i32) -> Self {
        match access {
            0 => RoomMetaAccess::Anonymous,
            1 => RoomMetaAccess::User,
            2 => RoomMetaAccess::Moderator,
            3 => RoomMetaAccess::Owner,
            4 => RoomMetaAccess::Admin,
            5 => RoomMetaAccess::System,
            _ => RoomMetaAccess::default()
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum MetaType<T: Default + Debug + PartialEq + Clone> {
    Null,
    Number(f64, T),
    String(String, T),
    Bool(bool, T)
}

impl<'de, T: Default + Debug + PartialEq + Clone + From<i32>> Deserialize<'de> for MetaType<T> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_any(MetaVisitor::default())
    }
}

impl<T: Default + Debug + PartialEq + Clone> Serialize for MetaType<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            MetaType::Null => serializer.serialize_none(),
            MetaType::Number(number, _) => serializer.serialize_f64(*number),
            MetaType::String(string, _) => serializer.serialize_str(string.as_str()),
            MetaType::Bool(boolean, _) => serializer.serialize_bool(*boolean),
        }
    }
}

#[derive(Default)]
struct MetaVisitor<T: Default + Debug + PartialEq + Clone + From<i32>> {
    _marker: PhantomData<T>
}


impl<'de, T: Default + Debug + PartialEq + Clone + From<i32>> Visitor<'de> for MetaVisitor<T> {
    type Value = MetaType<T>;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("meta type")
    }

    fn visit_some<D>(self, deserializer: D) -> Result<Self::Value, D::Error> where D: Deserializer<'de> {
        deserializer.deserialize_any(MetaVisitor::default())
    }

    fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E> where E: de::Error {
        Ok(MetaType::Number(value as f64, T::default()))
    }

    fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E> where E: de::Error {
        Ok(MetaType::Number(value as f64, T::default()))
    }

    fn visit_f64<E>(self, value: f64) -> Result<Self::Value, E> where E: de::Error {
        Ok(MetaType::Number(value, T::default()))
    }

    fn visit_f32<E>(self, value: f32) -> Result<Self::Value, E> where E: de::Error {
        Ok(MetaType::Number(value as f64, T::default()))
    }

    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E> where E: de::Error {
        Ok(MetaType::String(value.to_string(), T::default()))
    }

    fn visit_bool<E>(self, value: bool) -> Result<Self::Value, E> where E: de::Error {
        Ok(MetaType::Bool(value, T::default()))
    }

    fn visit_unit<E>(self) -> Result<Self::Value, E> where E: de::Error { 
        Ok(MetaType::Null)
    }

    fn visit_map<E>(self, mut access: E) -> Result<Self::Value, E::Error> where E: MapAccess<'de> {
        let mut visibility: Option<T> = None;
        let mut value: Option<serde_json::Value> = None;
        while let Some(key) = access.next_key::<&str>()? {
            match key {
                "access" => visibility = Some(match access.next_value::<usize>() {
                    Ok(n) => (n as i32).into(),
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
                        Ok(MetaType::Number(number.as_f64().unwrap_or_default(), visibility))
                    } else if number.is_i64() {
                        Ok(MetaType::Number(number.as_i64().unwrap_or_default() as f64, visibility))
                    } else if number.is_u64() {
                        Ok(MetaType::Number(number.as_u64().unwrap_or_default() as f64, visibility))
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
        // User meta data
        let s = "2";
        let deserialized: MetaType<UserMetaAccess> = serde_json::from_str(s).unwrap();
        assert_eq!(deserialized, MetaType::Number(2.0, UserMetaAccess::Anonymous));

        let s = "\"erhan\"";
        let deserialized: MetaType<UserMetaAccess> = serde_json::from_str(s).unwrap();
        assert_eq!(deserialized, MetaType::String("erhan".to_string(), UserMetaAccess::Anonymous));

        let s = "true";
        let deserialized: MetaType<UserMetaAccess> = serde_json::from_str(s).unwrap();
        assert_eq!(deserialized, MetaType::Bool(true, UserMetaAccess::Anonymous));

        let s = "false";
        let deserialized: MetaType<UserMetaAccess> = serde_json::from_str(s).unwrap();
        assert_eq!(deserialized, MetaType::Bool(false, UserMetaAccess::Anonymous));

        let s = "10.5";
        let deserialized: MetaType<UserMetaAccess>= serde_json::from_str(s).unwrap();
        assert_eq!(deserialized, MetaType::Number(10.5, UserMetaAccess::Anonymous));

        let s = "null";
        let deserialized: MetaType<UserMetaAccess>= serde_json::from_str(s).unwrap();
        assert_eq!(deserialized, MetaType::Null);

        // Room meta data
        let s = "2";
        let deserialized: MetaType<RoomMetaAccess> = serde_json::from_str(s).unwrap();
        assert_eq!(deserialized, MetaType::Number(2.0, RoomMetaAccess::Anonymous));

        let s = "\"erhan\"";
        let deserialized: MetaType<RoomMetaAccess> = serde_json::from_str(s).unwrap();
        assert_eq!(deserialized, MetaType::String("erhan".to_string(), RoomMetaAccess::Anonymous));

        let s = "true";
        let deserialized: MetaType<RoomMetaAccess> = serde_json::from_str(s).unwrap();
        assert_eq!(deserialized, MetaType::Bool(true, RoomMetaAccess::Anonymous));

        let s = "false";
        let deserialized: MetaType<RoomMetaAccess> = serde_json::from_str(s).unwrap();
        assert_eq!(deserialized, MetaType::Bool(false, RoomMetaAccess::Anonymous));

        let s = "10.5";
        let deserialized: MetaType<RoomMetaAccess>= serde_json::from_str(s).unwrap();
        assert_eq!(deserialized, MetaType::Number(10.5, RoomMetaAccess::Anonymous));

        let s = "null";
        let deserialized: MetaType<RoomMetaAccess>= serde_json::from_str(s).unwrap();
        assert_eq!(deserialized, MetaType::Null);
    }

    #[test]
    fn dict_deserialization() {
        // User meta data
        let s = r#"{"access": 3, "value": true}"#;
        let deserialized: MetaType<UserMetaAccess>= serde_json::from_str(s).unwrap();
        assert_eq!(deserialized, MetaType::Bool(true, UserMetaAccess::Me));

        let s = r#"{"access": 4, "value": "erhan"}"#;
        let deserialized: MetaType<UserMetaAccess>= serde_json::from_str(s).unwrap();
        assert_eq!(deserialized, MetaType::String("erhan".to_string(), UserMetaAccess::Mod));

        let s = r#"{"access": 0, "value": true}"#;
        let deserialized: MetaType<UserMetaAccess>= serde_json::from_str(s).unwrap();
        assert_eq!(deserialized, serde_json::from_str("true").unwrap());

        // Room meta data
        let s = r#"{"access": 3, "value": true}"#;
        let deserialized: MetaType<RoomMetaAccess>= serde_json::from_str(s).unwrap();
        assert_eq!(deserialized, MetaType::Bool(true, RoomMetaAccess::Owner));

        let s = r#"{"access": 4, "value": "erhan"}"#;
        let deserialized: MetaType<RoomMetaAccess>= serde_json::from_str(s).unwrap();
        assert_eq!(deserialized, MetaType::String("erhan".to_string(), RoomMetaAccess::Admin));

        let s = r#"{"access": 0, "value": true}"#;
        let deserialized: MetaType<RoomMetaAccess>= serde_json::from_str(s).unwrap();
        assert_eq!(deserialized, serde_json::from_str("true").unwrap());
    }

    #[test]
    fn wrong_deserialization() {
        // User meta data
        let s = r#"{"access": 3}"#;
        assert_eq!(serde_json::from_str::<MetaType<UserMetaAccess>>(s).err().unwrap().to_string(), r#""value" key is missing at line 1 column 13"#);

        let s = r#"{"value": true}"#;
        assert_eq!(serde_json::from_str::<MetaType<UserMetaAccess>>(s).err().unwrap().to_string(), r#""access" key is missing at line 1 column 15"#);

        let s = r#"{"access": 0, "value": true, "wrong": 1}"#;
        assert_eq!(serde_json::from_str::<MetaType<UserMetaAccess>>(s).err().unwrap().to_string(), r#""wrong" is not valid at line 1 column 36"#);

        let s = r#"{"access": "0", "value": true}"#;
        assert_eq!(serde_json::from_str::<MetaType<UserMetaAccess>>(s).err().unwrap().to_string(), r#"Invalid "access" type at line 1 column 14"#);

        let s = r#"{"access": 0, "value": {}}"#;
        assert_eq!(serde_json::from_str::<MetaType<UserMetaAccess>>(s).err().unwrap().to_string(), r#"Only, number, string and bool types are valid for "value" at line 1 column 26"#);


        // Room meta data
        let s = r#"{"access": 3}"#;
        assert_eq!(serde_json::from_str::<MetaType<RoomMetaAccess>>(s).err().unwrap().to_string(), r#""value" key is missing at line 1 column 13"#);

        let s = r#"{"value": true}"#;
        assert_eq!(serde_json::from_str::<MetaType<RoomMetaAccess>>(s).err().unwrap().to_string(), r#""access" key is missing at line 1 column 15"#);

        let s = r#"{"access": 0, "value": true, "wrong": 1}"#;
        assert_eq!(serde_json::from_str::<MetaType<RoomMetaAccess>>(s).err().unwrap().to_string(), r#""wrong" is not valid at line 1 column 36"#);

        let s = r#"{"access": "0", "value": true}"#;
        assert_eq!(serde_json::from_str::<MetaType<RoomMetaAccess>>(s).err().unwrap().to_string(), r#"Invalid "access" type at line 1 column 14"#);

        let s = r#"{"access": 0, "value": {}}"#;
        assert_eq!(serde_json::from_str::<MetaType<RoomMetaAccess>>(s).err().unwrap().to_string(), r#"Only, number, string and bool types are valid for "value" at line 1 column 26"#);
    }
}