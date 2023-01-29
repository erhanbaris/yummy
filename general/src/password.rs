use secrecy::*;
use serde::{
    de::{self, Visitor},
    Deserialize, Deserializer, Serialize, Serializer,
};

#[derive(Debug)]
pub struct Password(Secret<String>);

impl Password {
    pub fn get(&self) -> &String {
        self.0.expose_secret()
    }

    pub fn from(str: String) -> Self {
        Self(Secret::<String>::new(str))
    }
}

impl From<&str> for Password {
    fn from(str: &str) -> Self {
        Password::from(str.to_string())
    }
}

impl Serialize for Password {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.get())
    }
}

impl<'de> Deserialize<'de> for Password {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct StringVisitor;
        impl<'de> Visitor<'de> for StringVisitor {
            type Value = Password;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("an text")
            }

            fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(Password(SecretString::new(v)))
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(Password(SecretString::new(v.to_string())))
            }
        }

        deserializer.deserialize_string(StringVisitor)
    }
}
