"""Specification Validator."""

import argparse
import sys

import rich
from pydantic import ValidationError
from rich.table import Table
from rich_argparse import RichHelpFormatter

from spec.signed_doc import SignedDoc


def parse_args() -> argparse.Namespace:
    """Initialise and run the CLI parser."""
    parser = argparse.ArgumentParser(
        description="Project Catalyst - Architectural Specifications Validator",
        formatter_class=RichHelpFormatter,
    )
    parser.add_argument("spec", help="Path to JSON Specification file")

    return parser.parse_args()


def main() -> None:
    """Validate Architectural Specifications."""
    args = parse_args()

    # Get the compiled documentation json file
    # Specs are validated automatically on load.
    try:
        _spec = SignedDoc.load(args.spec)
        rich.print("Architectural Specifications Validated Successfully.")
    except ValidationError as exc:
        table = Table(
            title=f"{exc.error_count()} Locations where Schema Data does not match the {exc.title} Model.",
            caption="Model does not match Schema and needs updating.",
        )
        table.add_column("Key", no_wrap=True, style="yellow")
        table.add_column("Error", no_wrap=True, style="red")
        table.add_column("Input", no_wrap=True, max_width=30, style="grey37")

        error_links: dict[str, str] = {}
        errors = exc.errors()
        errors.sort(key=lambda x: [x["loc"], x["type"]])
        for error in errors:
            error_links[error["msg"]] = error["url"]  # type: ignore  # noqa: PGH003

            loc: list[str] = []
            for x in error["loc"]:
                if isinstance(x, int):
                    loc.append(f"[{x}]")
                else:
                    loc.append(f"{x}")

            table.add_row(
                ".".join(loc),
                error["msg"],
                str(error["input"]).splitlines()[0],
            )
        rich.print(table)

        for msg, url in error_links.items():
            rich.print(f"* {msg} : {url}")

        sys.exit(1)

    except Exception:  # noqa: BLE001
        rich.get_console().print_exception(show_locals=True)
        sys.exit(1)


if __name__ == "__main__":
    main()
