# ruff: noqa: D100, D102, D103, D107, RET505, PLR2004

from cryptography.hazmat.primitives.asymmetric.ed25519 import Ed25519PrivateKey
from pycardano.crypto.bip32 import BIP32ED25519PrivateKey


class Ed25519Keys:
    def __init__(self, sk_hex: str) -> None:
        self.sk_hex = sk_hex

    def pk_hex(self) -> str:
        if len(self.sk_hex) > 64:
            sk = BIP32ED25519PrivateKey(bytes.fromhex(self.sk_hex)[:64], bytes.fromhex(self.sk_hex)[64:])
            return sk.public_key.hex()
        else:
            sk = Ed25519PrivateKey.from_private_bytes(bytes.fromhex(self.sk_hex))
            return sk.public_key().public_bytes_raw().hex()

    def sign(self, msg: bytes) -> bytes:
        if len(self.sk_hex) > 64:
            sk = BIP32ED25519PrivateKey(bytes.fromhex(self.sk_hex)[:64], bytes.fromhex(self.sk_hex)[64:])
            return sk.sign(msg)
        else:
            sk = Ed25519PrivateKey.from_private_bytes(bytes.fromhex(self.sk_hex))
            return sk.sign(msg)
