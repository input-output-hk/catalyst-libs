"""CDDL Definition Specification."""

import typing
from collections import Counter
from functools import cached_property

from pydantic import BaseModel, ConfigDict, PrivateAttr, RootModel


class DocCluster(BaseModel):
    """Document Cluster Deserialized Specification."""

    docs: list[str]
    _name: str = PrivateAttr(default="Unknown")

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

    @cached_property
    def name(self) -> str:
        """Name."""
        return self._name


class DocClusters(RootModel[dict[str, DocCluster]]):
    """All Document Clusters."""

    root: dict[str, DocCluster]

    def model_post_init(self, context: typing.Any) -> None:  # noqa: ANN401
        """Extra setup after we deserialize."""
        super().model_post_init(context)

        # Set the name in each cluster.
        for cluster, value in self.root.items():
            value.set_name(cluster)

    def for_ref(self, ref: list[str]) -> DocCluster | None:
        """Get the cluster a document is in, if any."""
        for cluster in self.root.values():
            if cluster.is_cluster(ref):
                return cluster
        return None

    def get(self, doc_name: str) -> DocCluster | None:
        """Is the named document in a cluster."""
        for cluster in self.root.values():
            if cluster.is_in_cluster(doc_name):
                return cluster
        return None

    def name(self, doc_name: str) -> str | None:
        """Is the named document in a cluster of what name."""
        for cluster in self.root.values():
            if cluster.is_in_cluster(doc_name):
                return cluster.name
        return None
