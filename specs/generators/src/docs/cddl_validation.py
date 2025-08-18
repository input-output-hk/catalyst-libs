"""Generate CDDL Files."""

import argparse

from spec.signed_doc import SignedDoc

from .doc_generator import DocGenerator


class CDDLBlueprint(DocGenerator):
    """CDDL Blueprint Generator."""

    def __init__(self, args: argparse.Namespace, spec: SignedDoc) -> None:
        """Initialize."""
        super().__init__(args, spec, flags=self.NO_FLAGS, template="cddl/blueprint.cue.jinja")

    def generate(self) -> bool:
        """Generate the blueprint.cue."""
        self.generate_from_page_template()

        return super().generate()


class CDDLEarthfile(DocGenerator):
    """CDDL Earthfile Generator."""

    def __init__(self, args: argparse.Namespace, spec: SignedDoc) -> None:
        """Initialize."""
        super().__init__(args, spec, flags=self.NO_FLAGS, template="cddl/Earthfile.jinja")

    def generate(self) -> bool:
        """Generate the Earthfile."""
        blueprint = CDDLBlueprint(self._args, self._spec)
        blueprint_ok = blueprint.save_or_validate()

        self.generate_from_page_template()

        return super().generate() and blueprint_ok
