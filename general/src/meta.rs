use serde::{Serialize, de::Visitor, de::MapAccess, Deserialize, Deserializer, Serializer};
use std::{fmt::{self, Debug}, marker::PhantomData};
use serde::de::{self};
use serde::ser::SerializeSeq;
use serde_repr::{Serialize_repr, Deserialize_repr};

#[derive(Debug, PartialEq, Eq, Clone, Default, Serialize_repr, Deserialize_repr)]
#[repr(u8)]
pub enum MetaAction {
    #[default]
    OnlyAddOrUpdate = 0,
    RemoveUnusedMetas = 1,
    RemoveAllMetas = 2
}

impl TryFrom<i32> for MetaAction {
    type Error = &'static str;
    fn try_from(value: i32) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(MetaAction::OnlyAddOrUpdate),
            1 => Ok(MetaAction::RemoveUnusedMetas),
            2 => Ok(MetaAction::RemoveAllMetas),
            _ => Err("MetaAction value is not valid")
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Default, Serialize_repr, Deserialize_repr)]
#[repr(u8)]
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

#[derive(Debug, PartialEq, PartialOrd, Eq, Copy, Clone, Default, Serialize_repr, Deserialize_repr)]
#[repr(u8)]
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
#[repr(u8)]
pub enum MetaType<T: Default + Debug + PartialEq + Clone + From<i32>> {
    Null,
    Number(f64, T),
    String(String, T),
    Bool(bool, T),
    List(Box<Vec<MetaType<T>>>, T)
}

impl<T: Default + Debug + PartialEq + Clone + From<i32>> MetaType<T> {
    pub fn get_access_level(&self) -> T {
        match self {
            MetaType::Null => T::default(),
            MetaType::Number(_, access_level) => access_level.clone(),
            MetaType::String(_, access_level) => access_level.clone(),
            MetaType::Bool(_, access_level) => access_level.clone(),
            MetaType::List(_, access_level) => access_level.clone(),
        }
    }
}

impl<'de, T: Default + Debug + PartialEq + Clone + From<i32> + Into<i32>> Deserialize<'de> for MetaType<T> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_any(MetaVisitor::default())
    }
}

impl<T: Default + Debug + PartialEq + Clone + From<i32>> Serialize for MetaType<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            MetaType::Null => serializer.serialize_none(),
            MetaType::Number(number, _) => serializer.serialize_f64(*number),
            MetaType::String(string, _) => serializer.serialize_str(string.as_str()),
            MetaType::Bool(boolean, _) => serializer.serialize_bool(*boolean),
            MetaType::List(list, _) => {
                let mut seq = serializer.serialize_seq(Some(list.len()))?;
                for element in list.iter() {
                    seq.serialize_element(element)?;
                }
                seq.end()
            },
        }
    }
}

#[derive(Default)]
struct MetaVisitor<T: Default + Debug + PartialEq + Clone + From<i32> + Into<i32>> {
    _marker: PhantomData<T>
}


impl<'de, T: Default + Debug + PartialEq + Clone + From<i32> + Into<i32>> Visitor<'de> for MetaVisitor<T> {
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

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
        where
            A: de::SeqAccess<'de>, {
        let mut vec = Vec::new();

        while let Ok(Some(elem)) = seq.next_element() {
            vec.push(elem);
        }

        Ok(MetaType::List(Box::new(vec), T::default()))
    }

    fn visit_map<E>(self, mut access: E) -> Result<Self::Value, E::Error> where E: MapAccess<'de> {
        let mut visibility: Option<T> = None;
        let mut value: Option<MetaType<T>> = None;
        while let Some(key) = access.next_key::<&str>()? {
            match key {
                "access" => visibility = Some(match access.next_value::<usize>() {
                    Ok(n) => (n as i32).into(),
                    _ => return Err(de::Error::custom(r#"Invalid "access" type"#))
                }),
                "value" => value = Some(access.next_value::<MetaType<T>>()?),
                _ => return Err(de::Error::custom(format!(r#""{}" is not valid"#, key)))
            }
        }

        let visibility = match visibility {
            Some(visibility) => visibility,
            None => return Err(de::Error::custom(r#""access" key is missing"#))
        };
        
        match value {
            Some(value) => Ok(match value {
                MetaType::Null => MetaType::Null,
                MetaType::Number(val, _) => MetaType::Number(val, visibility),
                MetaType::String(val, _) => MetaType::String(val, visibility),
                MetaType::Bool(val, _) => MetaType::Bool(val, visibility),
                MetaType::List(val, _) => MetaType::List(val, visibility),
            }),
            None => Err(de::Error::custom(r#""value" key is missing"#))
        }
    }
}
