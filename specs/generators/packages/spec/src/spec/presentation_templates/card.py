"""Presentation Template Card Definition."""

from pydantic import BaseModel, ConfigDict, PrivateAttr, computed_field


class Card(BaseModel):
    """Specification of an individual Presentation Template Card."""

    name: str
    description: str
    available_docs: list[str]
    _card_id: str = PrivateAttr(default="Unknown")

    model_config = ConfigDict(extra="forbid")

    @computed_field
    @property
    def card_id(self) -> str:
        """Name Of the Element."""
        return self._card_id

    def set_card_id(self, val: str) -> None:
        """Set Card Id."""
        self._card_id = val
