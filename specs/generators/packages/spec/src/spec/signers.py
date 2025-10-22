"""Signers Specification."""

from pydantic import BaseModel, ConfigDict, Field, Enum


class Collaborators(str, Enum):
    """Signders Collaborators Specification."""

    collaborators_field_based = "collaborators"
    ref_field_based = "ref"
    excluded = "excluded"


class AllowedRoles(BaseModel):
    """Allowed Roles Specification."""

    user: list[str]
    admin: list[str] = Field(default=[])

    model_config = ConfigDict(extra="forbid")


class AllowedUpdaters(BaseModel):
    """Allowed Updaters Specification."""

    collaborators: Collaborators
    author: bool = Field(default=True)

    model_config = ConfigDict(extra="forbid")


class Signers(BaseModel):
    """Signers Specification."""

    roles: AllowedRoles
    referenced: bool = Field(default=False)
    update: dict[str, bool]

    model_config = ConfigDict(extra="forbid")
