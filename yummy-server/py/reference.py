import yummy


def pre_deviceid_auth(model: yummy.model.DeviceIdAuth):
    pass


def post_deviceid_auth(model: yummy.model.DeviceIdAuth, success: bool):
    pass


def pre_email_auth(model: yummy.model.EmailAuth):
    pass


def post_email_auth(model: yummy.model.EmailAuth, success: bool):
    pass


def pre_customid_auth(model: yummy.model.CustomIdAuth):
    pass


def post_customid_auth(model: yummy.model.CustomIdAuth, success: bool):
    pass


def pre_logout(model: yummy.model.Logout):
    pass


def post_logout(model: yummy.model.Logout, success: bool):
    pass


def pre_user_connected(model: yummy.model.UserConnected):
    pass


def post_user_connected(model: yummy.model.UserConnected, success: bool):
    pass


def pre_user_disconnected(model: yummy.model.UserDisconnected):
    pass


def post_user_disconnected(model: yummy.model.UserDisconnected, success: bool):
    pass


def pre_refresh_token(model: yummy.model.RefreshToken):
    pass


def post_refresh_token(model: yummy.model.RefreshToken, success: bool):
    pass


def pre_get_user_information(model: yummy.model.GetUserInformation):
    pass


def post_get_user_information(model: yummy.model.GetUserInformation, success: bool):
    pass
