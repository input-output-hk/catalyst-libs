#!/usr/bin/env -S uv run
"""Autogenerate Documentation Pages from the specification."""

import argparse
import sys
from pathlib import Path

from rich_argparse import RichHelpFormatter

from gen.docs_page_md import gen_docs_page_md
from gen.metadata_md import MetadataMd
from gen.spec_index import SpecIndex
from gen.spec_md import SpecMd
from gen.types_md import TypesMd
from spec.signed_doc import SignedDocSpec


def check_is_dir(base_path: Path) -> bool:
    """Check if the path exists, and is a directory.

    Otherwise try and make it.
    Fails if it exists and is NOT a directory.
    """
    # Check the base path exists and is a directory.
    if not base_path.exists():
        base_path.mkdir(parents=True)
    elif not base_path.is_dir():
        print("Base path is not a Directory")
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


def main(args: argparse.Namespace) -> None:
    """Generate Signed Document Specification documentation."""
    # Get the compiled documentation json file
    spec = SignedDocSpec(args.spec)

    # We start out hoping everything is OK.
    good = True

    # Generate each of the files.
    good &= SpecIndex(args, spec).save_or_validate()
    good &= SpecMd(args, spec).save_or_validate()
    good &= TypesMd(args, spec).save_or_validate()
    good &= MetadataMd(args, spec).save_or_validate()

    if not good:
        print("File Comparisons Failed, Documentation is not current.")
        sys.exit(1)

    if args.generate:
        print("Documentation Generated Successfully.")
    else:
        print("Documentation Validated Successfully.")


if __name__ == "__main__":
    main(parse_args())
