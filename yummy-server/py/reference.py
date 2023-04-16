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


def pre_update_room(model: model.UpdateRoom):
    pass


def post_update_room(model: model.UpdateRoom, success: bool):
    pass


def pre_join_to_room(model: model.UpdateRoom):
    pass


def post_join_to_room(model: model.JoinToRoom, success: bool):
    pass


def pre_process_waiting_user(model: model.ProcessWaitingUser):
    pass


def post_process_waiting_user(model: model.ProcessWaitingUser, success: bool):
    pass


def pre_kick_user_from_room(model: model.KickUserFromRoom):
    pass


def post_kick_user_from_room(model: model.KickUserFromRoom, success: bool):
    pass


def pre_disconnect_from_room(model: model.DisconnectFromRoom):
    pass


def post_disconnect_from_room(model: model.DisconnectFromRoom, success: bool):
    pass


def pre_message_to_room(model: model.MessageToRoom):
    pass


def post_message_to_room(model: model.MessageToRoom, success: bool):
    pass


def pre_room_list_request(model: model.RoomListRequest):
    pass


def post_room_list_request(model: model.RoomListRequest, success: bool):
    pass


def pre_waiting_room_joins(model: model.WaitingRoomJoins):
    pass


def post_waiting_room_joins(model: model.WaitingRoomJoins, success: bool):
    pass


def pre_get_room_request(model: model.GetRoomRequest):
    pass


def post_get_room_request(model: model.GetRoomRequest, success: bool):
    pass


def pre_play(model: model.Play):
    pass


def post_play(model: model.Play, success: bool):
    pass
