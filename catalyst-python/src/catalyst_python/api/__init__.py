import os

CAT_API_URL = os.environ["CAT_API_URL"]

CAT_INTERNAL_API_KEY = os.environ["CAT_INTERNAL_API_KEY"]


def cat_api_endpoint_url(endpoint: str):
    return f"{CAT_API_URL}/{endpoint}"
