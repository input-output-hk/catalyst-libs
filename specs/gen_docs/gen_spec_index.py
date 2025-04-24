"""Generate the spec.md file."""

import argparse

from doc_generator import DocGenerator
from signed_doc_spec import SignedDocSpec


class SpecIndex(DocGenerator):
    """Spec Index Generator."""

    def __init__(self, args: argparse.Namespace, spec: SignedDocSpec) -> None:
        """Initialize."""
        super().__init__(args, spec, ".pages", flags=self.NO_FLAGS)

    def generate(self) -> bool:
        """Generate the Spec Index."""
        self._filedata = """
title: Catalyst Signed Document
nav:
  - Specification: spec.md
  - Metadata Fields: metadata.md
  - Document Types: types.md
  - docs
"""
        return super().generate()
