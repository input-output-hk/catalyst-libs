from catalyst_python.ed25519 import Ed25519Keys
from catalyst_python.rbac_token import (
    generate_rbac_auth_token,
    generate_cat_id,
)


class AdminKey:
    def __init__(self, key: Ed25519Keys):
        self.key = key

    def cat_id(self) -> str:
        return generate_cat_id(
            scheme="admin.catalyst",
            network="cardano",
            subnet="preprod",
            role_0_key=self.key,
        )

    def auth_token(self) -> str:
        return generate_rbac_auth_token(
            scheme="admin.catalyst",
            network="cardano",
            subnet="preprod",
            role_0_key=self.key,
            signing_key=self.key,
        )
