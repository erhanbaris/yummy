from typing import NoReturn, Optional, Tuple

MetaType = int | str | bool | list['MetaType']


def fail(message: str) -> NoReturn:
    """ Throw exception with error message. Message will be sent to client. """
    ...


def get_user_meta(user_id: str, key: str) -> Optional[MetaType]:
    """
    Get user's meta information with key.
    """
    ...


def set_user_meta(user_id: str, key: str, value: MetaType, access_level: Optional[int]) -> bool:
    """
    Set user's meta information with key.
    """
    ...


def get_user_metas(user_id: str) -> dict[str, MetaType]:
    """
    Get user's all meta informations.
    """
    ...


def remove_user_meta(user_id: str, key: str):
    """
    Remove user's meta.
    """
    ...


def remove_user_metas(user_id: str) -> bool:
    """
    Remove all user's metas.
    """
    ...


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


class CustomIdAuth(BaseModel):
    def get_custom_id(self) -> str: ...
    def set_custom_id(self, custom_id: str): ...


class Logout(BaseModel):
    ...


class UserConnected:
    def get_user_id(self) -> Optional[str]: ...


class RefreshToken(BaseModel):
    ...


class GetUserInformation(BaseModel):
    def get_query_type(self) -> str: ...
    def get_requester_user_id(self) -> Optional[str]: ...
    def get_value(self) -> Tuple[Optional[str], Optional[str]]: ...
