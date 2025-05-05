"""CDDL Definition Specification."""

from collections import Counter

from pydantic import BaseModel, ConfigDict, PrivateAttr


class DocCluster(BaseModel):
    """Document Cluster Deserialized Specification."""

    docs: list[str]
    _name: str | None = PrivateAttr(default=None)

    model_config = ConfigDict(extra="forbid")

    def is_cluster(self, match: list[str]) -> bool:
        """Is this list of strings matching this cluster."""
        return Counter(self.docs) == Counter(match)

    def is_in_cluster(self, match: str) -> bool:
        """Is this doc in this cluster."""
        return match in self.docs

    def set_name(self, name: str) -> None:
        """Set the clusters name."""
        self._name = name
