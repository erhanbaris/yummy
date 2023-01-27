use std::{cell::RefCell, rc::Rc};

use crate::auth::model::EmailAuthRequest;

pub trait YummyAuthInterface {
    fn pre_email_auth<'a>(&self, _model: Rc<RefCell<EmailAuthRequest>>) -> anyhow::Result<()> { Ok(()) }
    fn post_email_auth<'a>(&self, _model: Rc<RefCell<EmailAuthRequest>>, _successed: bool) -> anyhow::Result<()> { Ok(()) }
}
