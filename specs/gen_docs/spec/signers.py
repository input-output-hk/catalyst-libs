"""Signers Specification."""

from pydantic import BaseModel, ConfigDict, Field


class AllowedRoles(BaseModel):
    """Allowed Roles Specification."""

    user: list[str]
    admin: list[str] = Field(default=[])

    model_config = ConfigDict(extra="forbid")


class AllowedUpdaters(BaseModel):
    """Allowed Updaters Specification."""

    collaborators: bool = Field(default=False)
    author: bool = Field(default=True)
    any: bool = Field(default=False)

    model_config = ConfigDict(extra="forbid")


class Signers(BaseModel):
    """Signers Specification."""

    roles: AllowedRoles
    referenced: bool = Field(default=False)
    update: AllowedUpdaters

    model_config = ConfigDict(extra="forbid")
