#!/usr/bin/env -S uv run
"""Autogenerate Documentation Pages from the specification."""

import argparse
import sys
from pathlib import Path

import rich
from pydantic import ValidationError
from rich_argparse import RichHelpFormatter

from docs.doc_index import DocIndex
from docs.presentation_template_md import PresentationTemplatesMd
from spec.signed_doc import SignedDoc

from .form_templates_md import FormTemplatesMd
from .metadata_md import MetadataMd
from .spec_index import SpecIndex
from .spec_md import SpecMd
from .types_md import TypesMd


def check_is_dir(base_path: Path) -> bool:
    """Check if the path exists, and is a directory.

    Otherwise try and make it.
    Fails if it exists and is NOT a directory.
    """
    # Check the base path exists and is a directory.
    if not base_path.exists():
        base_path.mkdir(parents=True)
    elif not base_path.is_dir():
        rich.print("Base path is not a Directory")
        return False
    return True


def parse_args() -> argparse.Namespace:
    """Initialise and run the CLI parser."""
    parser = argparse.ArgumentParser(
        description="Markdown Documentation Generator for the Signed Documentation Specifications",
        formatter_class=RichHelpFormatter,
    )
    parser.add_argument("spec", help="Path to JSON Specification file")
    parser.add_argument(
        "-o",
        "--output",
        help="Where the docs are generated/located (directory)",
        required=True,
    )
    parser.add_argument(
        "-g",
        "--generate",
        action="store_true",
        help="Set to cause docs to be generated, otherwise they are validated",
    )

    args = parser.parse_args()

    # Check the base path exists and is a directory.
    if not check_is_dir(Path(args.output)):
        sys.exit(1)
    if not check_is_dir(Path(args.output, "docs")):
        sys.exit(1)

    return args


def main() -> None:
    """Generate Signed Document Specification documentation."""
    args = parse_args()

    # Get the compiled documentation json file
    try:
        spec = SignedDoc.load(args.spec)
    except ValidationError as e:
        SignedDoc.validation_error(e)
        sys.exit(1)
    except Exception:  # noqa: BLE001
        rich.get_console().print_exception(show_locals=True)
        sys.exit(1)

    # We start out hoping everything is OK.
    good = True

    # Generate each of the TOP Level files.
    # Each top level file is responsible for generating lower level
    # files they require or provide the primary reference for, and so on.
    good &= SpecIndex(args, spec).save_or_validate()
    good &= SpecMd(args, spec).save_or_validate()
    good &= TypesMd(args, spec).save_or_validate()
    good &= MetadataMd(args, spec).save_or_validate()
    good &= DocIndex(args, spec).save_or_validate()
    good &= FormTemplatesMd(args, spec).save_or_validate()
    good &= PresentationTemplatesMd(args, spec).save_or_validate()

    if not good:
        rich.print("File Comparisons Failed, Documentation is not current.")
        sys.exit(1)

    if args.generate:
        rich.print("Documentation Generated Successfully.")
    else:
        rich.print("Documentation Validated Successfully.")


if __name__ == "__main__":
    main()
