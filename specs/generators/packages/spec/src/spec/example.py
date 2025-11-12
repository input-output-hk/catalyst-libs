"""Payload Specification."""

import json
import textwrap
from typing import Any

from pydantic import Base64Bytes, BaseModel, ConfigDict


class JsonExample(BaseModel):
    """An Example of the payload."""

    title: str
    description: str
    example: dict[str, Any]

    model_config = ConfigDict(extra="forbid")

    @classmethod
    def default(cls) -> list[JsonExample]:
        """Return Default list."""
        return []

    def __str__(self) -> str:
        """Get the example properly formatted as markdown."""
        example = json.dumps(self.example, indent=2, sort_keys=True)
        textwrap.indent(example, "    ")

        return f"""

<!-- markdownlint-disable MD013 MD046 max-one-sentence-per-line -->
??? example "Example: {self.title}"

{textwrap.indent(self.description, "    ")}

    ```json
{textwrap.indent(example, "    ")}
    ```

<!-- markdownlint-enable MD013 MD046 max-one-sentence-per-line -->
""".strip()


class CborExample(BaseModel):
    """An Example of the payload."""

    title: str
    description: str
    example: Base64Bytes

    model_config = ConfigDict(extra="forbid")

    @classmethod
    def default(cls) -> list[CborExample]:
        """Return Default list."""
        return []

    def __str__(self) -> str:
        """Get the example properly formatted as markdown."""
        example = json.dumps(self.example, indent=2, sort_keys=True)
        textwrap.indent(example, "    ")

        return f"""

<!-- markdownlint-disable MD013 MD046 max-one-sentence-per-line -->
??? example "Example: {self.title}"

{textwrap.indent(self.description, "    ")}

    ```json
{textwrap.indent(example, "    ")}
    ```

<!-- markdownlint-enable MD013 MD046 max-one-sentence-per-line -->
""".strip()
