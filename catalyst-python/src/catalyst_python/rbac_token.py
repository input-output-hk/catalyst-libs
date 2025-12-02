from catalyst_python.catalyst_id import generate_cat_id
from catalyst_python.ed25519 import Ed25519Keys
from catalyst_python.utils import base64_url


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
