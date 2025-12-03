# ruff: noqa: D100, D103, I001, S113

import requests

from catalyst_python.api import cat_api_endpoint_url


# cardano assets GET
def assets(stake_address: str, slot_no: int, token: str | None = None) -> requests.Response:
    url = cat_api_endpoint_url(f"api/v1/cardano/assets/{stake_address}?asat=SLOT:{slot_no}")
    headers = {
        "Content-Type": "application/json",
    }

    if token is not None:
        headers["Authorization"] = f"Bearer {token}"

    return requests.get(url, headers=headers)
