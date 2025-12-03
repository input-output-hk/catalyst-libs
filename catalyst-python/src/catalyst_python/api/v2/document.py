# ruff: noqa: D100, D103, A002, S113

import requests

from catalyst_python.api import cat_api_endpoint_url

URL = cat_api_endpoint_url("api/v2/document")


# Signed document POST
def post(filter: dict, limit: str | None = None, page: str | None = None):
    headers = {"Content-Type": "application/json"}
    url = f"{URL}/index"
    query_params = []
    if limit is not None:
        query_params.append(f"limit={limit}")
    if page is not None:
        query_params.append(f"page={page}")

    if query_params:
        url += "?" + "&".join(query_params)
    return requests.post(url, headers=headers, json=filter)
