from datetime import datetime, timezone
import base64
from enum import IntEnum
from ed25519 import Ed25519Keys


class RoleID(IntEnum):
    ROLE_0 = 0
    PROPOSER = 3

    def __str__(self):
        return f"{int(self)}"


# Default is set to URI format
# Optional field = subnet, role id, rotation, username, nonce
def generate_cat_id(
    network: str,
    role_0_key: Ed25519Keys,
    scheme: str | None = None,
    subnet: str | None = None,
    role_id: RoleID | None = None,
    rotation: str | None = None,
    username: str | None = None,
    nonce: str | None = None,
) -> str:
    role0_pk_b64 = base64_url(bytes.fromhex(role_0_key.pk_hex()))

    # If nonce is set to none, use current timestamp
    # If set to empty string, use empty string (no nonce)
    if nonce is None:
        nonce = f"{int(datetime.now(timezone.utc).timestamp())}"

    # Authority part
    authority = ""
    if username:
        authority += f"{username}"
    if nonce:
        authority += f":{nonce}"
    authority += "@"

    if subnet:
        authority += f"{subnet}.{network}"
    else:
        authority += network

    # Path
    path = f"{role0_pk_b64}"
    if role_id is not None:
        path += f"/{role_id}"
        if rotation is not None:
            path += f"/{rotation}"

    if scheme:
        return f"{scheme}://{authority}/{path}"
    else:
        return f"{authority}/{path}"


def generate_rbac_auth_token(
    network: str,
    subnet: str,
    signing_key: Ed25519Keys,
    role_0_key: Ed25519Keys,
    scheme: str | None = None,
    sig: str | None = None,
    username: str | None = None,
    nonce: str | None = None,
    cat_id: str | None = None,
) -> str:
    token_prefix = "catid."
    if cat_id is None:
        cat_id = f"{token_prefix}{
            generate_cat_id(
                scheme=scheme, network=network, subnet=subnet, username=username, role_0_key=role_0_key, nonce=nonce
            )
        }."
    else:
        cat_id = f"{token_prefix}{cat_id}."

    if sig is None:
        signature = signing_key.sign(cat_id.encode())
    else:
        signature = sig.encode()

    signature_b64 = base64_url(signature)

    return f"{cat_id}{signature_b64}"


def base64_url(data: bytes) -> str:
    # URL safety and no padding base 64
    return base64.urlsafe_b64encode(data).decode().rstrip("=")
