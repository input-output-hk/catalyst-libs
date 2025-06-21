"""Metadata Formats Specification."""

from pydantic import BaseModel, ConfigDict, RootModel


class MetadataFormat(BaseModel):
    """Metadata Formats Deserialized Specification."""

    description: str
    cddl: str

    model_config = ConfigDict(extra="forbid")


class MetadataFormats(RootModel[dict[str, MetadataFormat]]):
    """Metadata Formats Deserialized Specification."""

    root: dict[str, MetadataFormat]

    @property
    def all(self) -> list[str]:
        """Get names of all metadata formats."""
        return list(self.root.keys())

    def get(self, name: str) -> MetadataFormat:
        """Get named metadata format."""
        return self.root[name]
