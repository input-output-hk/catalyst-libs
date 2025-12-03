# ruff: noqa: D100, D101, D102, D103, D107, S101, S603, PLW1510, I001

from typing import Any
from enum import StrEnum
import copy
import os
import time
import subprocess
import json
from tempfile import NamedTemporaryFile

from catalyst_python.admin import AdminKey
from catalyst_python.catalyst_id import RoleID
from catalyst_python.uuid import uuid_v7
from catalyst_python.ed25519 import Ed25519Keys
from catalyst_python.rbac_chain import RBACChain


class DocType(StrEnum):
    brand_parameters = "3e4808cc-c86e-467b-9702-d60baa9d1fca"
    brand_parameters_form_template = "fd3c1735-80b1-4eea-8d63-5f436d97ea31"
    campaign_parameters = "0110ea96-a555-47ce-8408-36efe6ed6f7c"
    campaign_parameters_form_template = "7e8f5fa2-44ce-49c8-bfd5-02af42c179a3"
    category_parameters = "48c20109-362a-4d32-9bba-e0a9cf8b45be"
    category_parameters_form_template = "65b1e8b0-51f1-46a5-9970-72cdf26884be"
    comment_moderation_action = "84a4b502-3b7e-47fd-84e4-6fee08794bd7"
    contest_delegation = "764f17fb-cc50-4979-b14a-b213dbac5994"
    contest_parameters = "788ff4c6-d65a-451f-bb33-575fe056b411"
    contest_parameters_form_template = "08a1e16d-354d-4f64-8812-4692924b113b"
    presentation_template = "cb99b9bd-681a-49d8-9836-89107c02e8ef"
    proposal = "7808d2ba-d511-40af-84e8-c0d1625fdfdc"
    proposal_comment = "b679ded3-0e7c-41ba-89f8-da62a17898ea"
    proposal_comment_form_template = "0b8424d4-ebfd-46e3-9577-1775a69d290c"
    proposal_form_template = "0ce8ab38-9258-4fbc-a62e-7faa6e58318f"
    proposal_moderation_action = "a552451a-8e5b-409d-83a0-21eac26bbf8c"
    proposal_submission_action = "5e60e623-ad02-4a1b-a1ac-406db978ee48"
    rep_nomination = "bf9abd97-5d1f-4429-8e80-740fea371a9c"
    rep_nomination_form_template = "431561a5-9c2b-4de1-8e0d-78eb4887e35d"
    rep_profile = "0f2c86a2-ffda-40b0-ad38-23709e1c10b3"
    rep_profile_form_template = "564cbea3-44d3-4303-b75a-d9fdda7e5a80"


class SignedDocumentBase:
    def __init__(
        self,
        metadata: dict[str, Any],
        content: dict[str, Any],
        cat_id: str,
        key: Ed25519Keys,
    ) -> None:
        self.metadata = metadata
        self.content = content
        self.cat_id = cat_id
        self.key = key

    def new_version(self) -> None:
        time.sleep(1)
        self.metadata["ver"] = uuid_v7()


class SignedDocument(SignedDocumentBase):
    def copy(self) -> SignedDocument:
        return SignedDocument(
            metadata=copy.deepcopy(self.metadata),
            content=copy.deepcopy(self.content),
            cat_id=copy.deepcopy(self.cat_id),
            key=copy.deepcopy(self.key),
        )

    # Build and sign document, returns hex str of document bytes
    def build_and_sign(
        self,
    ) -> str:
        with (
            NamedTemporaryFile() as metadata_file,
            NamedTemporaryFile() as doc_content_file,
            NamedTemporaryFile() as signed_doc_file,
        ):
            mk_signed_doc_path = os.environ["MK_SIGNED_DOC_PATH"]
            json_str = json.dumps(self.metadata)
            metadata_file.write(json_str.encode(encoding="utf-8"))
            metadata_file.flush()

            json_str = json.dumps(self.content)
            doc_content_file.write(json_str.encode(encoding="utf-8"))
            doc_content_file.flush()

            subprocess.run(
                [
                    mk_signed_doc_path,
                    "build",
                    doc_content_file.name,
                    signed_doc_file.name,
                    metadata_file.name,
                ],
                capture_output=True,
            )

            subprocess.run(
                [
                    mk_signed_doc_path,
                    "sign",
                    signed_doc_file.name,
                    self.key.sk_hex,
                    self.cat_id,
                ],
                capture_output=True,
            )

            return signed_doc_file.read().hex()


# ------------------- #
# Signed Docs Factory #
# ------------------- #


def proposal_doc(
    content: dict[str, Any],
    proposal_form_template_doc: SignedDocumentBase,
    param_doc: SignedDocumentBase,
    rbac_chain: RBACChain,
) -> SignedDocument:
    metadata = __create_metadata(
        doc_type=DocType.proposal,
        content_type="application/json",
        template=proposal_form_template_doc,
        parameters=[param_doc],
    )

    (cat_id, key) = rbac_chain.cat_id_for_role(RoleID.PROPOSER)
    return SignedDocument(metadata, content, cat_id, key)


def proposal_form_template_doc(
    content: dict[str, Any],
    param_doc: SignedDocumentBase,
    admin_key: AdminKey,
) -> SignedDocument:
    metadata = __create_metadata(
        doc_type=DocType.proposal_form_template,
        content_type="application/schema+json",
        parameters=[param_doc],
    )

    return SignedDocument(metadata, content, admin_key.cat_id(), admin_key.key)


def category_parameters_doc(
    content: dict[str, Any],
    category_parameters_form_template_doc: SignedDocumentBase,
    param_doc: SignedDocumentBase,
    admin_key: AdminKey,
) -> SignedDocumentBase:
    metadata = __create_metadata(
        doc_type=DocType.category_parameters,
        content_type="application/json",
        template=category_parameters_form_template_doc,
        parameters=[param_doc],
    )
    return SignedDocument(metadata, content, admin_key.cat_id(), admin_key.key)


def category_parameters_form_template_doc(
    content: dict[str, Any],
    param_doc: SignedDocumentBase,
    admin_key: AdminKey,
) -> SignedDocumentBase:
    metadata = __create_metadata(
        doc_type=DocType.category_parameters_form_template,
        content_type="application/schema+json",
        parameters=[param_doc],
    )
    return SignedDocument(metadata, content, admin_key.cat_id(), admin_key.key)


def campaign_parameters_doc(
    content: dict[str, Any],
    campaign_parameters_form_template_doc: SignedDocumentBase,
    param_doc: SignedDocumentBase,
    admin_key: AdminKey,
) -> SignedDocumentBase:
    metadata = __create_metadata(
        doc_type=DocType.campaign_parameters,
        content_type="application/json",
        template=campaign_parameters_form_template_doc,
        parameters=[param_doc],
    )
    return SignedDocument(metadata, content, admin_key.cat_id(), admin_key.key)


def campaign_parameters_form_template_doc(
    content: dict[str, Any],
    param_doc: SignedDocumentBase,
    admin_key: AdminKey,
) -> SignedDocumentBase:
    metadata = __create_metadata(
        doc_type=DocType.campaign_parameters_form_template,
        content_type="application/schema+json",
        parameters=[param_doc],
    )
    return SignedDocument(metadata, content, admin_key.cat_id(), admin_key.key)


def brand_parameters_doc(
    content: dict[str, Any],
    brand_parameters_form_template_doc: SignedDocumentBase,
    admin_key: AdminKey,
) -> SignedDocumentBase:
    metadata = __create_metadata(
        doc_type=DocType.brand_parameters,
        content_type="application/json",
        template=brand_parameters_form_template_doc,
    )
    return SignedDocument(metadata, content, admin_key.cat_id(), admin_key.key)


def brand_parameters_form_template_doc(content: dict[str, Any], admin_key: AdminKey) -> SignedDocumentBase:
    metadata = __create_metadata(
        doc_type=DocType.brand_parameters_form_template,
        content_type="application/schema+json",
    )
    return SignedDocument(metadata, content, admin_key.cat_id(), admin_key.key)


def __create_metadata(
    doc_type: DocType,
    content_type: str,
    template: SignedDocumentBase | None = None,
    parameters: list[SignedDocumentBase] | None = None,
) -> dict[str, Any]:
    doc_id = uuid_v7()

    metadata: dict[str, Any] = {
        "content-encoding": "br",
        "content-type": content_type,
        "id": doc_id,
        "ver": doc_id,
        "type": doc_type,
    }

    if template is not None:
        metadata["template"] = {
            "id": template.metadata["id"],
            "ver": template.metadata["ver"],
            "cid": "0x",
        }
    if parameters is not None:
        metadata["parameters"] = [{"id": p.metadata["id"], "ver": p.metadata["ver"], "cid": "0x"} for p in parameters]

    return metadata
