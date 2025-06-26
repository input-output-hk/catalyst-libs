"""Markdown Helper and Formatting Functions."""


class MarkdownHelpers:
    """Markdown Helper and Formatting Functions."""

    @staticmethod
    def format_link(name: str, depth: int = 0, *, file: str = "metadata.md", monospace: bool = False) -> str:
        """Format link."""
        link = f"{file}#{name.lower().replace(' ', '-')}"

        while depth > 0:
            link = f"../{link}"
            depth -= 1

        if monospace:
            name = f"`{name}`"

        return f"[{name}]({link})"

    @staticmethod
    def doc_ref_link(name: str, depth: int = 0, *, html: bool = False) -> str:
        """Metadata Document Reference link."""
        link = name.lower().replace(" ", "_")

        if html:
            link += "/"
        else:
            link += ".md"

        if depth == 0:
            link = f"./docs/{link}"
        else:
            maxdepth = 0 if html else 1
            while depth > maxdepth:
                link = f"../{link}"
                depth -= 1

        if html:
            return link
        return f"[{name}]({link})"

    @staticmethod
    def field_link(name: str, depth: int = 0) -> str:
        """Metadata Field link."""
        return MarkdownHelpers.format_link(name, depth, monospace=True)
