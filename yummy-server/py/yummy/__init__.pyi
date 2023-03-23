import constants
import model
import user

from typing import NoReturn


def fail(message: str) -> NoReturn:
    """ Throw exception with error message. Message will be sent to client. """
    ...


__all__ = ["constants", "model", "user"]
