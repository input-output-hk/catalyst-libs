"""Generate the spec.md file."""

import argparse

from spec.signed_doc import SignedDoc

from .doc_generator import DocGenerator


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
  - Document Templates: templates.md
  - docs
"""
        return super().generate()
