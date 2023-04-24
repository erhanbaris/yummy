/* **************************************************************************************************************** */
/* **************************************************** MODS ****************************************************** */
/* *************************************************** IMPORTS **************************************************** */
/* **************************************************************************************************************** */
use std::borrow::Cow;

use actix_web::{error::{JsonPayloadError, InternalError}, HttpRequest, HttpResponse};
use serde::{Deserialize, Serialize, de::DeserializeOwned};

/* **************************************************************************************************************** */
/* ******************************************** STATICS/CONSTS/TYPES ********************************************** */
/* **************************************************** MACROS **************************************************** */
/* *************************************************** STRUCTS **************************************************** */
/* **************************************************************************************************************** */
#[derive(Debug, Deserialize, Serialize)]
pub struct Answer<'a> {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request_id: Option<usize>,
    pub status: bool,

    #[serde(rename = "type")]
    pub response_type: Cow<'a, str>
}

#[derive(Debug, Deserialize, Serialize)]
pub struct GenericAnswer<'a, T>
{
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request_id: Option<usize>,

    #[serde(default)]
    pub status: bool,
    
    #[serde(rename = "type")]
    pub response_type: Cow<'a, str>,

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
impl<'a> Answer<'a> {
    pub fn success(request_id: Option<usize>, response_type: Cow<'a, str>) -> Self {
        Self {
            request_id,
            status: true,
            response_type
        }
    }
    
    pub fn fail(request_id: Option<usize>, response_type: Cow<'a, str>) -> Self {
        Self {
            request_id,
            status: false,
            response_type
        }
    }
}

/* **************************************************************************************************************** */
/* ********************************************** TRAIT IMPLEMENTS ************************************************ */
/* **************************************************************************************************************** */
impl<'a> From<Answer<'a>> for String {
    fn from(source: Answer) -> Self {
        serde_json::to_string(&source).unwrap()
    }
}

impl<'a, T> GenericAnswer<'a, T>
where T: Serialize {
    pub fn success(request_id: Option<usize>, response_type: Cow<'a, str>, result: T) -> Self {
        Self {
            request_id,
            status: true,
            response_type,
            result
        }
    }
    
    pub fn fail(request_id: Option<usize>, response_type: Cow<'a, str>, result: T) -> GenericAnswer<ErrorResponse<T>> {
        GenericAnswer {
            request_id,
            status: false,
            response_type,
            result: ErrorResponse { error: result}
        }
    }
}

impl<'a, T: Serialize> From<GenericAnswer<'_, T>> for String {
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

impl<'a, T: DeserializeOwned> From<String> for GenericAnswer<'_, T> {
    fn from(source: String) -> Self {
        serde_json::from_str(&source).unwrap()
    }
}

/* **************************************************************************************************************** */
/* ************************************************* MACROS CALL ************************************************** */
/* ************************************************** UNIT TESTS ************************************************** */
/* **************************************************************************************************************** */
