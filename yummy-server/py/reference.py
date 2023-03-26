from yummy import model


def pre_deviceid_auth(model: model.DeviceIdAuth):
    pass


def post_deviceid_auth(model: model.DeviceIdAuth, success: bool):
    pass


def pre_email_auth(model: model.EmailAuth):
    pass


def post_email_auth(model: model.EmailAuth, success: bool):
    pass


def pre_customid_auth(model: model.CustomIdAuth):
    pass


def post_customid_auth(model: model.CustomIdAuth, success: bool):
    pass


def pre_logout(model: model.Logout):
    pass


def post_logout(model: model.Logout, success: bool):
    pass


def pre_user_connected(model: model.UserConnected):
    pass


def post_user_connected(model: model.UserConnected, success: bool):
    pass


def pre_user_disconnected(model: model.UserDisconnected):
    pass


def post_user_disconnected(model: model.UserDisconnected, success: bool):
    pass


def pre_refresh_token(model: model.RefreshToken):
    pass


def post_refresh_token(model: model.RefreshToken, success: bool):
    pass


def pre_get_user_information(model: model.GetUserInformation):
    pass


def post_get_user_information(model: model.GetUserInformation, success: bool):
    pass


def pre_update_user(model: model.UpdateUser):
    pass


def post_update_user(model: model.UpdateUser, success: bool):
    pass


def pre_create_room(model: model.CreateRoom):
    pass


def post_create_room(model: model.CreateRoom, success: bool):
    pass
