# ruff: noqa: D100, D103

from catalyst_python.api import cat_api_endpoint_url

LIVE_URL = "api/v1/health/live"
READY_URL = "api/v1/health/ready"


def live() -> str:
    return cat_api_endpoint_url(LIVE_URL)


def ready() -> str:
    return cat_api_endpoint_url(READY_URL)
