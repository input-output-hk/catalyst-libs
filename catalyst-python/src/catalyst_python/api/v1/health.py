from api import cat_api_endpoint_url
LIVE_URL = "api/v1/health/live"
READY_URL = "api/v1/health/ready"

def live():
    return cat_api_endpoint_url(LIVE_URL)

def ready():
    return cat_api_endpoint_url(READY_URL)
