from typing import Optional

from yummy import MetaType


def get_user_meta(user_id: str, key: str) -> Optional[MetaType]:
    """
    Get user's meta information with key.
    """
    ...

def get_user_meta_access(user_id: str, key: str) -> Optional[int]:
    """
    Get user's meta type information with key.
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
