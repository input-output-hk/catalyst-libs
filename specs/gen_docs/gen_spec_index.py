# Generate the spec.md file

from doc_generator import DocGenerator


class SpecIndex(DocGenerator):
    def __init__(self, args, spec):
        super().__init__(args, spec, ".pages", flags=self.NO_FLAGS)

    def generate(self):
        self._filedata = """
title: Catalyst Signed Document
nav:
  - Specification: spec.md
  - Metadata Fields: metadata.md
  - Document Types: types.md
  - docs
"""
        super().generate()
