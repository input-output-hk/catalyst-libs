"""Specification Validator."""

import argparse
import sys

import rich
from pydantic import ValidationError
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
        SignedDoc.validation_error(exc)
        sys.exit(1)

    except Exception:  # noqa: BLE001
        rich.get_console().print_exception(show_locals=True)
        sys.exit(1)


if __name__ == "__main__":
    main()
