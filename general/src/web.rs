use serde::{Deserialize, Serialize, de::DeserializeOwned};

#[derive(Debug, Deserialize, Serialize)]
pub struct Answer {
    pub status: bool,
}


impl Answer {
    pub fn success() -> Self {
        Self { status: true }
    }
    
    pub fn fail() -> Self {
        Self { status: false }
    }
    
    pub fn new(status: bool) -> Self {
        Self { status }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct GenericAnswer<T> {
    pub status: bool,
    pub result: Option<T>,
}

impl<T> GenericAnswer<T>
where T: DeserializeOwned + Serialize {
    pub fn success(result: T) -> Self {
        Self {
            status: true,
            result: Some(result)
        }
    }
    
    pub fn fail(result: T) -> Self {
        Self {
            status: false,
            result: Some(result)
        }
    }
    
    pub fn new(status: bool, result: T) -> Self {
        Self {
            status,
            result: Some(result)
        }
    }
}

impl<T: Serialize> From<GenericAnswer<T>> for String {
    fn from(source: GenericAnswer<T>) -> Self {
        serde_json::to_string(&source).unwrap_or_default()
    }
}

impl<T: DeserializeOwned> From<String> for GenericAnswer<T> {
    fn from(source: String) -> Self {
        serde_json::from_str(&source).unwrap()
    }
}
