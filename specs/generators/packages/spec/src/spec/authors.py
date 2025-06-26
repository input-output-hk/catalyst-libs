"""Change Log Entry Specification."""

from pydantic import RootModel


class Authors(RootModel[dict[str, str]]):
    """Authors of the Specifications."""

    root: dict[str, str]  # name: email

    def combine(self, other: "Authors") -> "Authors":
        """Combine Two Authors lists into a single Authors List."""
        combined = self.root | other.root
        return self.model_validate(combined)

    def email(self, name: str) -> str:
        """Get Email for authors name."""
        return self.root.get(name, "Unknown")

    def all(self) -> list[str]:
        """Get All Authors."""
        return sorted(self.root.keys())
