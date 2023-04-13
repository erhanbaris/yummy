from typing import Optional

from yummy import MetaType


def get_room_meta(room_id: str, key: str) -> Optional[MetaType]:
    """
    Get room's meta information with key.
    """
    ...


def set_room_meta(room_id: str, key: str, value: MetaType, access_level: Optional[int]) -> bool:
    """
    Set room's meta information with key.
    """
    ...


def get_room_metas(room_id: str) -> dict[str, MetaType]:
    """
    Get room's all meta informations.
    """
    ...


def remove_room_meta(room_id: str, key: str):
    """
    Remove room's meta.
    """
    ...


def remove_room_metas(room_id: str) -> bool:
    """
    Remove all room's metas.
    """
    ...


def message_to_room(room_id: str, message: any) -> bool:
    """
    Send message to room users
    """
    ...


def message_to_room_user(room_id: str, user_id: str, message: any) -> bool:
    """
    Send message to room user
    """
    ...
