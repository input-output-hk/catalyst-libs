"""Generate the spec.md file."""

import argparse

from gen.doc_generator import DocGenerator
from spec.signed_doc import SignedDoc


class SpecIndex(DocGenerator):
    """Spec Index Generator."""

    def __init__(self, args: argparse.Namespace, spec: SignedDoc) -> None:
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
