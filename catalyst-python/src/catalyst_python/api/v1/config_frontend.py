# ruff: noqa: D100, D103, I001, S113

import requests

from catalyst_python.api import cat_api_endpoint_url, CAT_INTERNAL_API_KEY

URL = cat_api_endpoint_url("api/v1/config/frontend")


def put(json_config: dict) -> requests.Response:
    headers = {
        "Content-Type": "application/json",
        "X-API-Key": CAT_INTERNAL_API_KEY,
    }
    return requests.put(URL, headers=headers, json=json_config)


def get() -> requests.Response:
    return requests.get(URL)
