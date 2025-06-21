"""Documentation Links."""

import typing

from pydantic import BaseModel, ConfigDict, Field, HttpUrl, RootModel, computed_field


class LinkAKA(RootModel[dict[str, str]]):
    """Link AKA."""

    root: dict[str, str]


class Links(RootModel[dict[str, HttpUrl]]):
    """Documentation Links."""

    root: dict[str, HttpUrl]

    _aka: LinkAKA

    def set_link_aka(self, aka: LinkAKA) -> None:
        """Associate the Link AKA with the main documentation Links."""
        self._aka = aka

    def aka(self, link_name: str) -> str | None:
        """Get a Link AKA for a link name, or None if it doesn't exist."""
        return self._aka.root.get(link_name)

    @computed_field
    @property
    def all(self) -> list[str]:
        """Get a list of ALL link names, including AKAs.

        Sorted from longest Link name to shortest.
        """
        link_aka: list[str] = list(self._aka.root.keys())
        primary_links: list[str] = list(self.root.keys())

        return sorted(link_aka + primary_links, key=lambda x: -len(x))

    def link(self, link_name: str) -> str:
        """Get a link for a link name."""
        return f"{self.root[link_name]}"


class Documentation(BaseModel):
    """Documentation Information."""

    links: Links
    link_aka: LinkAKA = Field(alias="linkAKA")

    model_config = ConfigDict(extra="forbid")

    def model_post_init(self, context: typing.Any) -> None:  # noqa: ANN401
        """Extra setup after we deserialize."""
        super().model_post_init(context)

        # Associate the Link AKA with documentation links.
        self.links.set_link_aka(self.link_aka)
