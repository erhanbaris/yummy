use general::auth::UserAuth;

pub enum ResponseExtra {
    None,
    Auth(UserAuth)
}

pub struct Response {
    pub message: String,
    pub extra: ResponseExtra
}
