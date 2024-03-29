/* **************************************************************************************************************** */
/* **************************************************** MODS ****************************************************** */
/* **************************************************************************************************************** */
pub mod collection;

/* **************************************************************************************************************** */
/* *************************************************** IMPORTS **************************************************** */
/* **************************************************************************************************************** */
use serde::{Serialize, de::Visitor, de::MapAccess, Deserialize, Deserializer, Serializer};
use std::{fmt::{self, Debug}, marker::PhantomData};
use serde::de::{self};
use serde::ser::SerializeSeq;
use serde_repr::{Serialize_repr, Deserialize_repr};

/* **************************************************************************************************************** */
/* ******************************************** STATICS/CONSTS/TYPES ********************************************** */
/* **************************************************************************************************************** */
pub type UserMetaType = MetaType<UserMetaAccess>;
pub type RoomMetaType = MetaType<RoomMetaAccess>;

/* **************************************************************************************************************** */
/* **************************************************** MACROS **************************************************** */
/* *************************************************** STRUCTS **************************************************** */
/* **************************************************************************************************************** */
#[derive(Default)]
struct MetaVisitor<T: Default + Debug + PartialEq + Clone + From<i32> + Into<i32>> {
    _marker: PhantomData<T>
}

/* **************************************************************************************************************** */
/* **************************************************** ENUMS ***************************************************** */
/* **************************************************************************************************************** */
#[derive(Debug, PartialEq, Eq, Clone, Default, Serialize_repr, Deserialize_repr)]
#[repr(u8)]
pub enum MetaAction {
    #[default]
    OnlyAddOrUpdate = 0,
    RemoveUnusedMetas = 1,
    RemoveAllMetas = 2
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

#[derive(Debug, PartialEq, Clone)]
#[repr(u8)]
pub enum MetaType<T: Default + Debug + PartialEq + Clone + From<i32>> {
    Null,
    Number(f64, T),
    String(String, T),
    Bool(bool, T),
    List(Box<Vec<MetaType<T>>>, T)
}

/* **************************************************************************************************************** */
/* ************************************************** FUNCTIONS *************************************************** */
/* *************************************************** TRAITS ***************************************************** */
/* ************************************************* IMPLEMENTS *************************************************** */
/* ********************************************** TRAIT IMPLEMENTS ************************************************ */
/* **************************************************************************************************************** */

impl From<i32> for MetaAction {
    fn from(value: i32) -> Self {
        match value {
            0 => MetaAction::OnlyAddOrUpdate,
            1 => MetaAction::RemoveUnusedMetas,
            2 => MetaAction::RemoveAllMetas,
            _ => MetaAction::default()
        }
    }
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

/* **************************************************************************************************************** */
/* ************************************************* MACROS CALL ************************************************** */
/* ************************************************** UNIT TESTS ************************************************** */
/* **************************************************************************************************************** */
#[cfg(test)]
mod test {
    use crate::meta::{MetaAction, UserMetaAccess, RoomMetaAccess};

    #[test]
    fn meta_action() {
        assert_eq!(MetaAction::from(0), MetaAction::OnlyAddOrUpdate);
        assert_eq!(MetaAction::from(1), MetaAction::RemoveUnusedMetas);
        assert_eq!(MetaAction::from(2), MetaAction::RemoveAllMetas);

        assert_eq!(MetaAction::from(-1), MetaAction::OnlyAddOrUpdate);
        assert_eq!(MetaAction::from(100), MetaAction::OnlyAddOrUpdate);
    }

    #[test]
    fn user_meta_access() {
        assert_eq!(UserMetaAccess::from(0), UserMetaAccess::Anonymous);
        assert_eq!(UserMetaAccess::from(1), UserMetaAccess::User);
        assert_eq!(UserMetaAccess::from(2), UserMetaAccess::Friend);
        assert_eq!(UserMetaAccess::from(3), UserMetaAccess::Me);
        assert_eq!(UserMetaAccess::from(4), UserMetaAccess::Mod);
        assert_eq!(UserMetaAccess::from(5), UserMetaAccess::Admin);
        assert_eq!(UserMetaAccess::from(6), UserMetaAccess::System);

        assert_eq!(UserMetaAccess::from(-1), UserMetaAccess::Anonymous);
        assert_eq!(UserMetaAccess::from(100), UserMetaAccess::Anonymous);
    }

    #[test]
    fn room_meta_access() {
        assert_eq!(RoomMetaAccess::from(0), RoomMetaAccess::Anonymous);
        assert_eq!(RoomMetaAccess::from(1), RoomMetaAccess::User);
        assert_eq!(RoomMetaAccess::from(2), RoomMetaAccess::Moderator);
        assert_eq!(RoomMetaAccess::from(3), RoomMetaAccess::Owner);
        assert_eq!(RoomMetaAccess::from(4), RoomMetaAccess::Admin);
        assert_eq!(RoomMetaAccess::from(5), RoomMetaAccess::System);

        assert_eq!(i32::from(RoomMetaAccess::Anonymous), 0);
        assert_eq!(i32::from(RoomMetaAccess::User), 1);
        assert_eq!(i32::from(RoomMetaAccess::Moderator), 2);
        assert_eq!(i32::from(RoomMetaAccess::Owner), 3);
        assert_eq!(i32::from(RoomMetaAccess::Admin), 4);
        assert_eq!(i32::from(RoomMetaAccess::System), 5);

        assert_eq!(RoomMetaAccess::from(-1), RoomMetaAccess::Anonymous);
        assert_eq!(RoomMetaAccess::from(100), RoomMetaAccess::Anonymous);
    }
}
