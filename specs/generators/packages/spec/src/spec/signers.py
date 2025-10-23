"""Signers Specification."""

from pydantic import BaseModel, ConfigDict, Field
from enum import Enum


class UpdatersType(str, Enum):
    """Allowed Updaters Types."""

    collaborators = "collaborators"
    ref_field_based = "ref"
    author = "author"


class AllowedRoles(BaseModel):
    """Allowed Roles Specification."""

    user: list[str]
    admin: list[str] = Field(default=[])

    model_config = ConfigDict(extra="forbid")


class AllowedUpdaters(BaseModel):
    """Allowed Updaters Specification."""

    type: UpdatersType
    description: str

    model_config = ConfigDict(extra="forbid")


class Signers(BaseModel):
    """Signers Specification."""

    roles: AllowedRoles
    update: AllowedUpdaters

    model_config = ConfigDict(extra="forbid")
