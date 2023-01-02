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

impl From<Answer> for String {
    fn from(source: Answer) -> Self {
        serde_json::to_string(&source).unwrap()
    }
}

impl From<String> for Answer {
    fn from(source: String) -> Self {
        serde_json::from_str(&source).unwrap()
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct GenericAnswer<T> {
    pub status: bool,

    #[serde(flatten)]
    pub result: T,
}

impl<T> GenericAnswer<T>
where T: Serialize {
    pub fn success(result: T) -> Self {
        Self {
            status: true,
            result: result
        }
    }
    
    pub fn fail(result: T) -> GenericAnswer<ErrorResponse<T>> {
        GenericAnswer {
            status: false,
            result: ErrorResponse { error: result}
        }
    }
    
    pub fn new(status: bool, result: T) -> Self {
        Self {
            status,
            result: result
        }
    }
}

impl<T: Serialize> From<GenericAnswer<T>> for String {
    fn from(source: GenericAnswer<T>) -> Self {
        match serde_json::to_string(&source) {
            Ok(data) => data,
            Err(error) => {
                println!("{}", error.to_string());
                String::new()
            }
        }
    }
}

impl<T: DeserializeOwned> From<String> for GenericAnswer<T> {
    fn from(source: String) -> Self {
        serde_json::from_str(&source).unwrap()
    }
}


#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ErrorResponse<T: Serialize> {
    pub error: T
}

impl<T: Serialize> From<ErrorResponse<T>> for String {
    fn from(source: ErrorResponse<T>) -> Self {
        serde_json::to_string(&source).unwrap()
    }
}
