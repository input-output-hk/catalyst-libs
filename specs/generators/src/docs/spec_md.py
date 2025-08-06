"""Generate the spec.md file."""

import argparse

import polars as pl
from great_tables import GT

from spec.cddl.cose import CoseHeader, HeaderType
from spec.signed_doc import SignedDoc

from .cddl_file import CDDLFile
from .doc_generator import DocGenerator


class SpecMd(DocGenerator):
    """Generate the spec.md file."""

    def __init__(self, args: argparse.Namespace, spec: SignedDoc) -> None:
        """Initialise Spec.md generator."""
        super().__init__(args, spec, template="spec.md.jinja")

    def header_parameter_doc(self, header: CoseHeader) -> str:
        """Create documentation for a single cose header."""
        custom_header = " Custom Header parameter label."
        if not isinstance(header.cose_label, str):
            custom_header = "COSE Standard header parameter label."

        table_data: dict[str, list[str]] = {"Group": [], "Headings": [], "Values": [], "Docs": []}

        table_data["Group"].append("Definition")
        table_data["Headings"].append("Required")
        table_data["Values"].append(header.required.value)
        table_data["Docs"].append("Is the field required?")

        table_data["Group"].append("Definition")
        table_data["Headings"].append("Cose Label")
        table_data["Values"].append(str(header.cose_label))
        table_data["Docs"].append(custom_header)

        table_data["Group"].append("Definition")
        table_data["Headings"].append("Format")
        table_data["Values"].append(header.format)
        table_data["Docs"].append(self._spec.cose.header_formats.get(header.format).description)

        if isinstance(header.value, list) and len(header.value) > 0:
            for value in header.value:
                if header.format == "Media Type":
                    description = self._spec.content_types.description(value)
                elif header.format == "HTTP Content Encoding":
                    description = self._spec.encoding_types.description(value)
                else:
                    description = header.format

                table_data["Group"].append("Supported Values")
                table_data["Headings"].append("")
                table_data["Values"].append(value)
                table_data["Docs"].append(description)

        params = pl.DataFrame(table_data)

        table = (
            GT(params)
            .with_id(id=f"spec {header.name()}".replace(" ", "_"))
            .tab_header(title=f"{header.name()}", subtitle=f"\n\n{header.description.split('.', maxsplit=1)[0]}\n\n")
            .fmt_markdown("Docs")
            .tab_stub(rowname_col="Headings", groupname_col="Group")
            .tab_options(column_labels_hidden=True, container_width="100%", table_width="100%")
            .opt_stylize(style=6)
        )

        return (
            f"""
#### `{header.name()}`

{header.description}

{self.wrap_html(table.as_raw_html())}
""".strip()
            + "\n"
        )

    def cose_header_parameters(self, header_type: HeaderType) -> str:
        """Insert details about Cose header Parameters that are defined for use."""
        if header_type == HeaderType.DOCUMENT:
            headers = self._spec.cose.headers.all
        elif header_type == HeaderType.SIGNATURE:
            headers = self._spec.cose.signature_headers.all
        else:
            return ""  # No Cose Headers in metadata.

        header_parameters_doc = ""
        for header in headers:
            header_parameters_doc += self.header_parameter_doc(header)

        return header_parameters_doc.strip()

    def generate(self) -> bool:
        """Generate a `spec.md` file from the definitions."""
        signed_doc_cddl = CDDLFile(self._args, self._spec, "signed_document")
        if not signed_doc_cddl.save_or_validate():
            return False

        self.generate_from_page_template(signed_doc_cddl=signed_doc_cddl, header_type=HeaderType)

        return super().generate()
