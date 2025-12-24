# ruff: noqa: D100, D103

import base64


def base64_url(data: bytes) -> str:
    return base64.urlsafe_b64encode(data).decode().rstrip("=")
