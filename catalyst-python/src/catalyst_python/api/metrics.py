# ruff: noqa: D100, D103

import requests
from catalyst_python.api import cat_api_endpoint_url


def metrics():
    return requests.get(cat_api_endpoint_url("metrics"))
