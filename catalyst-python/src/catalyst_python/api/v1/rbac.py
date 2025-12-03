# ruff: noqa: D100, D103, S113

import requests

from catalyst_python.api import cat_api_endpoint_url

URL = cat_api_endpoint_url("api/v1/rbac/registration")


def get(lookup: str | None, token: str, extra_headers: dict | None = None) -> requests.Response:
    headers = {
        "Authorization": f"Bearer {token}",
    }
    if extra_headers:
        headers.update(extra_headers)
    return requests.get(URL, headers=headers, params={"lookup": lookup})
