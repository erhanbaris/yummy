pub mod auth;
pub mod user;
pub mod room;
pub mod conn;
pub mod plugin;

mod macros;

pub trait YummyModel {
    fn get_request_type(&self) -> &'static str;
}