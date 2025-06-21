"""Document Types."""

from pydantic import RootModel

from spec.base_types import BaseTypes, DocTypeId


class DocType(RootModel[list[DocTypeId]]):
    """A Document Type."""

    root: list[DocTypeId]

    _base_types: BaseTypes

    def set_base_types(self, types: BaseTypes) -> None:
        """Save a local copy of the Base Types."""
        self._base_types = types

    def formatted_names(self, *, prefix: str = "", separator: str = "/", suffix: str = "") -> str:
        """Return a formatted doc types entry."""
        type_names: list[str] = [self._base_types.name(sub_type) for sub_type in self.root]
        return f"{prefix}{separator.join(type_names)}{suffix}"

    def formatted_ids(  # noqa: PLR0913
        self,
        *,
        prefix: str = "[",
        separator: str = ",",
        start_quote: str = "",
        end_quote: str = "",
        suffix: str = "]",
        cbor: bool = True,
    ) -> str:
        """Return doc types formatted optionally as cbor."""
        id_strings: list[str] = (
            [uuid.as_cbor for uuid in self.root] if cbor else [uuid.as_uuid_str for uuid in self.root]
        )
        id_strings = [f"{start_quote}{ids}{end_quote}" for ids in id_strings]
        return f"{prefix}{separator.join(id_strings)}{suffix}"
