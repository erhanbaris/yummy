/* **************************************************************************************************************** */
/* **************************************************** MODS ****************************************************** */
/* *************************************************** IMPORTS **************************************************** */
/* **************************************************************************************************************** */
use actix_web::{error::{JsonPayloadError, InternalError}, HttpRequest, HttpResponse};
use serde::{Deserialize, Serialize, de::DeserializeOwned};

/* **************************************************************************************************************** */
/* ******************************************** STATICS/CONSTS/TYPES ********************************************** */
/* **************************************************** MACROS **************************************************** */
/* *************************************************** STRUCTS **************************************************** */
/* **************************************************************************************************************** */
#[derive(Debug, Deserialize, Serialize)]
pub struct Answer {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request_id: Option<usize>,
    pub status: bool,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct GenericAnswer<T> {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request_id: Option<usize>,
    pub status: bool,

    #[serde(flatten)]
    pub result: T,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ErrorResponse<T: Serialize> {
    pub error: T
}

/* **************************************************************************************************************** */
/* **************************************************** ENUMS ***************************************************** */
/* ************************************************** FUNCTIONS *************************************************** */
/* **************************************************************************************************************** */
pub fn json_error_handler(err: JsonPayloadError, _: &HttpRequest) -> actix_web::Error {
    let detail = err.to_string();
    let res = HttpResponse::BadRequest().body("error");
    log::error!("Json parse issue: {}", detail);
    
    InternalError::from_response("Json format is not valid. Please check request definition.", res).into()
}

/* **************************************************************************************************************** */
/* *************************************************** TRAITS ***************************************************** */
/* ************************************************* IMPLEMENTS *************************************************** */
/* **************************************************************************************************************** */
impl Answer {
    pub fn success(request_id: Option<usize>) -> Self {
        Self { request_id, status: true }
    }
    
    pub fn fail(request_id: Option<usize>) -> Self {
        Self { request_id, status: false }
    }
}

/* **************************************************************************************************************** */
/* ********************************************** TRAIT IMPLEMENTS ************************************************ */
/* **************************************************************************************************************** */
impl From<Answer> for String {
    fn from(source: Answer) -> Self {
        serde_json::to_string(&source).unwrap()
    }
}

impl<T> GenericAnswer<T>
where T: Serialize {
    pub fn success(request_id: Option<usize>, result: T) -> Self {
        Self {
            request_id,
            status: true,
            result
        }
    }
    
    pub fn fail(request_id: Option<usize>, result: T) -> GenericAnswer<ErrorResponse<T>> {
        GenericAnswer {
            request_id,
            status: false,
            result: ErrorResponse { error: result}
        }
    }
}

impl<T: Serialize> From<GenericAnswer<T>> for String {
    fn from(source: GenericAnswer<T>) -> Self {
        match serde_json::to_string(&source) {
            Ok(data) => data,
            Err(error) => {
                println!("{}", error);
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

/* **************************************************************************************************************** */
/* ************************************************* MACROS CALL ************************************************** */
/* ************************************************** UNIT TESTS ************************************************** */
/* **************************************************************************************************************** */
