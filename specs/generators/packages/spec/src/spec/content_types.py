"""Content Types Specification."""

from pydantic import BaseModel, ConfigDict, Field, RootModel


class ContentType(BaseModel):
    """Content Types Deserialized Specification."""

    description: str
    coap_type: int | None = Field(default=None)

    model_config = ConfigDict(extra="forbid")


class ContentTypes(RootModel[dict[str, ContentType]]):
    """Content Types Deserialized Specification."""

    root: dict[str, ContentType]

    def description(self, name: str) -> str:
        """Get description of the Content Type."""
        return self.root[name].description


class EncodingType(BaseModel):
    """Encoding Types Deserialized Specification."""

    description: str

    model_config = ConfigDict(extra="forbid")


class EncodingTypes(RootModel[dict[str, EncodingType]]):
    """Content Types Deserialized Specification."""

    root: dict[str, EncodingType]

    def description(self, name: str) -> str:
        """Get description of the Encoding Type."""
        return self.root[name].description
