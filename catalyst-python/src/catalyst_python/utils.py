import base64


def base64_url(data: bytes) -> str:
    # URL safety and no padding base 64
    return base64.urlsafe_b64encode(data).decode().rstrip("=")
