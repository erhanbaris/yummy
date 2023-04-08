from typing import Optional, Tuple


MetaType = int | str | bool | list['MetaType']


class AuthModel:
    def get_user_id(self) -> Optional[str]: ...
    def get_session_id(self) -> Optional[str]: ...


class RequestIdModel:
    def get_request_id(self) -> Optional[int]: ...
    def set_request_id(self, request_id: int): ...


class BaseModel(AuthModel, RequestIdModel):
    ...


class DeviceIdAuth(BaseModel):
    def get_device_id(self) -> str: ...
    def set_device_id(self, device_id: str): ...


class EmailAuth(BaseModel):
    def get_email(self) -> str: ...
    def set_email(self, email: str): ...

    def get_password(self) -> str: ...
    def set_password(self, password: str): ...

    def get_if_not_exist_create(self) -> bool: ...
    def set_if_not_exist_create(self, value: bool): ...


class CustomIdAuth(BaseModel):
    def get_custom_id(self) -> str: ...
    def set_custom_id(self, custom_id: str): ...


class Logout(BaseModel):
    ...


class UserConnected:
    def get_user_id(self) -> Optional[str]: ...


class UserDisconnected(BaseModel):
    def get_send_message(self) -> bool: ...
    def set_send_message(self, send_message: bool): ...


class RefreshToken(BaseModel):
    ...


class GetUserInformation(BaseModel):
    def get_query_type(self) -> str: ...
    def get_requester_user_id(self) -> Optional[str]: ...
    def get_value(self) -> Tuple[Optional[str], Optional[str]]: ...


class UpdateUser(BaseModel):
    def get_target_user_id(self) -> Optional[str]: ...

    def get_name(self) -> Optional[str]: ...
    def set_name(self, value: Optional[str]): ...

    def get_email(self) -> Optional[str]: ...
    def set_email(self, value: Optional[str]): ...

    def get_password(self) -> Optional[str]: ...
    def set_password(self, value: Optional[str]): ...

    def get_device_id(self) -> Optional[str]: ...
    def set_device_id(self, value: Optional[str]): ...

    def get_custom_id(self) -> Optional[str]: ...
    def set_custom_id(self, value: Optional[str]): ...

    def get_user_type(self) -> Optional[int]: ...
    def set_user_type(self, value: Optional[int]): ...

    def get_meta_action(self) -> Optional[int]: ...
    def set_meta_action(self, value: Optional[int]): ...

    def get_metas(self) -> Optional[dict[str, MetaType]]: ...
    def set_metas(self, value: Optional[dict[str, MetaType]]): ...


class CreateRoom(BaseModel):
    def get_name(self) -> Optional[str]: ...
    def set_name(self, value: Optional[str]): ...

    def get_description(self) -> Optional[str]: ...
    def set_description(self, value: Optional[str]): ...

    def get_join_request(self) -> Optional[bool]: ...
    def set_join_request(self, value: Optional[bool]): ...

    def get_access_type(self) -> Optional[int]: ...
    def set_access_type(self, value: Optional[int]): ...

    def get_max_user(self) -> Optional[int]: ...
    def set_max_user(self, value: Optional[int]): ...

    def get_metas(self) -> Optional[dict[str, MetaType]]: ...
    def set_metas(self, value: Optional[dict[str, MetaType]]): ...

    def get_tags(self) -> Optional[list[str]]: ...
    def set_tags(self, value: Optional[list[str]]): ...


class UpdateRoom(BaseModel):
    def get_room_id(self) -> Optional[str]: ...

    def get_name(self) -> Optional[str]: ...
    def set_name(self, value: Optional[str]): ...

    def get_description(self) -> Optional[str]: ...
    def set_description(self, value: Optional[str]): ...

    def get_join_request(self) -> Optional[bool]: ...
    def set_join_request(self, value: Optional[bool]): ...

    def get_access_type(self) -> Optional[int]: ...
    def set_access_type(self, value: Optional[int]): ...

    def get_max_user(self) -> Optional[int]: ...
    def set_max_user(self, value: Optional[int]): ...

    def get_meta_action(self) -> Optional[int]: ...
    def set_meta_action(self, value: Optional[int]): ...

    def get_metas(self) -> Optional[dict[str, MetaType]]: ...
    def set_metas(self, value: Optional[dict[str, MetaType]]): ...

    def get_user_permission(self) -> Optional[dict[str, int]]: ...
    def set_user_permission(self, value: Optional[dict[str, int]]): ...

    def get_tags(self) -> Optional[list[str]]: ...
    def set_tags(self, value: Optional[list[str]]): ...


class JoinToRoom(BaseModel):
    def get_room_id(self) -> Optional[str]: ...

    def get_room_user_type(self) -> int: ...
    def set_room_user_type(self, value: int): ...


class ProcessWaitingUser(BaseModel):
    def get_room_id(self) -> Optional[str]: ...
    def get_target_user_id(self) -> Optional[str]: ...

    def get_status(self) -> bool: ...
    def set_status(self, value: bool): ...


class KickUserFromRoom(BaseModel):
    def get_room_id(self) -> Optional[str]: ...
    def get_target_user_id(self) -> Optional[str]: ...

    def get_ban(self) -> bool: ...
    def set_ban(self, value: bool): ...


class DisconnectFromRoom(BaseModel):
    def get_room_id(self) -> Optional[str]: ...


class MessageToRoom(BaseModel):
    def get_room_id(self) -> Optional[str]: ...

    def get_message(self) -> str: ...
    def set_message(self, value: str): ...


class RoomListRequest(RequestIdModel):
    def get_tag(self) -> Optional[str]: ...
    def set_tag(self, value: Optional[str]): ...

    def get_members(self) -> list[int]: ...
    def set_members(self, value: list[int]): ...


class WaitingRoomJoins(BaseModel):
    def get_room_id(self) -> Optional[str]: ...


class GetRoomRequest(BaseModel):
    def get_room_id(self) -> Optional[str]: ....

    def get_members(self) -> list[int]: ...
    def set_members(self, value: list[int]): ...
