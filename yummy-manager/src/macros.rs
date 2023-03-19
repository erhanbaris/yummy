#[macro_export]
macro_rules! get_user_id_from_auth {
    ($model: expr, $err: expr) => {
        match &$model.auth.deref() {
            Some(user) => &user.user,
            None => return $err
        }
    };
    ($model: expr) => {
        match &$model.auth.deref() {
            Some(user) => &user.user,
            None => return Err(anyhow::anyhow!(AuthError::TokenNotValid))
        }
    };
}

#[macro_export]
macro_rules! get_session_id_from_auth {
    ($model: expr, $err: expr) => {
        match &$model.auth.deref() {
            Some(user) => &user.session,
            None => return $err
        }
    };
    ($model: expr) => {
        match &$model.auth.deref() {
            Some(user) => &user.session,
            None => return Err(anyhow::anyhow!(AuthError::TokenNotValid))
        }
    };
}

#[macro_export]
macro_rules! get_user_session_id_from_auth {
    ($model: expr, $err: expr) => {
        match &$model.auth.deref() {
            Some(user) => (&user.user, &user.session),
            None => return $err
        }
    };
    ($model: expr) => {
        match &$model.auth.deref() {
            Some(user) => (&user.user, &user.session),
            None => return Err(anyhow::anyhow!(AuthError::TokenNotValid))
        }
    };
}