import json
from catalyst_python.admin import AdminKey
from catalyst_python.ed25519 import Ed25519Keys
from catalyst_python.catalyst_ffi import brand_parameters_form_template_doc


def test_simple_signed_doc():
    content = {}
    key = Ed25519Keys("4dc7e0388d106e973d602044daae8838b16bbae651a368d0b5c0d8ee990e09b4")
    admin = AdminKey(key=key, network="cardano")
    doc = brand_parameters_form_template_doc(
        content=json.dumps(content), sk=admin.key.sk_hex, kid=admin.cat_id(), id=None
    )
    assert doc.ref().id() == doc.id()
    assert doc.ref().ver() == doc.ver()
    assert doc.type() == "fd3c1735-80b1-4eea-8d63-5f436d97ea31"
