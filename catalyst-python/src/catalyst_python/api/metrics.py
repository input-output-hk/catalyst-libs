import requests
from api import cat_api_endpoint_url


def metrics():
    return requests.get(cat_api_endpoint_url("metrics"))
