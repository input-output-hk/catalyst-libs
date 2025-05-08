"""Generate the document relationship diagram files."""

import argparse
import textwrap

from gen.doc_generator import DocGenerator
from gen.graphviz_doc_diagram import Cluster, DotFile, DotLink, DotLinkEnd, DotSignedDoc, FontTheme, TableRow
from spec.metadata import Metadata
from spec.optional import OptionalField
from spec.signed_doc import HeaderType, SignedDoc


class DocRelationshipFile(DocGenerator):
    """Generate a Document Relationship Diagram File."""

    DIAGRAM_PATH: str = "diagrams/"

    def __init__(self, args: argparse.Namespace, spec: SignedDoc, doc_name: str | None = None, depth: int = 0) -> None:
        """Document Relationship Diagram File Generator."""
        file_name = doc_name.lower().replace(" ", "_").replace("-", "_") if doc_name is not None else "all"
        file_name = f"{self.DIAGRAM_PATH}{file_name}.dot"

        super().__init__(args, spec, file_name, flags=self.NO_FLAGS, depth=depth)
        self._document_name = doc_name

    def markdown_reference(
        self, *, indent: int = 0, relative_doc: DocGenerator | None = None, extension: str = "png"
    ) -> str:
        """Create a Markdown formatted reference for the DOT file."""
        file_path = self.file_path(relative_doc)
        file_name = self.file_name().rsplit("/")[-1]

        return textwrap.indent(
            f"""
<!-- markdownlint-disable max-one-sentence-per-line -->

```graphviz dot {file_name}.{extension}
{{{{ include_file('./{file_path}', indent={indent + 4}) }}}}
```

<!-- markdownlint-enable max-one-sentence-per-line -->
""".strip(),
            " " * indent,
        )

    def generate(self) -> bool:  # noqa: C901
        """Generate a Document Relationship Diagram File."""
        doc_names = self._spec.document_names() if self._document_name is None else [self._document_name]

        file_id = self._document_name if self._document_name is not None else "All"
        file_title = textwrap.fill(f"{file_id} Document Relationships", width=30)

        dot_file = DotFile(
            self._document_name, file_title, depth=self._depth, title_size=150 if self._document_name is None else 50
        )

        all_dst_refs: list[str] = []

        for doc in doc_names:
            cluster = Cluster.from_doc_cluster(self._spec.doc_in_cluster(doc))
            doc_table = DotSignedDoc(
                table_id=doc,
                title_href=Metadata.doc_ref_link(doc, self._depth, html=True),
                cluster=cluster,
            )
            doc_data = self._spec.get_document(doc)

            # Add content type explicitely to table.
            doc_table.add_row(TableRow(name="content type", value=doc_data.headers["content type"].value))

            # Add all used Metadata to table.
            for meta in self._spec.all_headers(HeaderType.METADATA):
                doc_metadata = self._spec.get_metadata(meta, doc)
                # Skip excluded metadata.
                if doc_metadata.required == OptionalField.excluded:
                    continue

                if meta == "type":
                    doc_table.add_row(
                        TableRow(
                            name=meta,
                            value=doc_data.type,
                            value_theme=FontTheme(face="Courier", bold=True, italic=True),
                        )
                    )
                    continue

                if doc_metadata.format == "Document Reference":
                    doc_type = doc_metadata.type
                    for link_dst in doc_type:
                        # If we link to ourselves,
                        # link through the top of the table (E to N)
                        # Creates better self references.
                        # Otherwise links flow from E to W
                        dst_dir = "n" if doc == link_dst else "w"

                        # Add dummy destination table, in case we don't have it in our docs.
                        ref_cluster = Cluster.from_doc_cluster(self._spec.doc_in_cluster(link_dst))
                        dummy_table = DotSignedDoc(
                            table_id=link_dst,
                            title_href=Metadata.doc_ref_link(
                                link_dst,
                                depth=self._depth,
                                html=True,
                            ),
                            cluster=ref_cluster,
                        )
                        dot_file.add_table(dummy_table)

                        dst_port = "title"
                        if ref_cluster is not None and ref_cluster != cluster:
                            dst_port = ref_cluster

                        dot_file.add_link(
                            DotLink(
                                src=DotLinkEnd(id=doc, port=meta),
                                dst=DotLinkEnd(id=link_dst, port=dst_port, dir=dst_dir),
                            )
                        )
                        all_dst_refs.append(link_dst)
                    if len(doc_type) == 0:
                        doc_type = ["Unspecified"]
                    doc_table.add_row(TableRow(name=meta, value=doc_type))
                    continue

                # Everything else
                doc_table.add_row(TableRow(name=meta, value=doc_metadata.format))
            dot_file.add_table(doc_table)

            # If we are referenced by any doc thats not in our doc list, create a dummy doc and link.
            for ref_doc in doc_data.all_docs_referencing:
                if ref_doc not in doc_names:
                    # Then we need to create a dummy doc and link.
                    ref_cluster = Cluster.from_doc_cluster(self._spec.doc_in_cluster(ref_doc))
                    ref_doc_table = DotSignedDoc(
                        table_id=ref_doc,
                        title_href=Metadata.doc_ref_link(ref_doc, self._depth, html=True),
                        cluster=ref_cluster,
                    )
                    dot_file.add_table(ref_doc_table)
                    dst_port = "title"
                    if cluster is not None and ref_cluster != cluster:
                        dst_port = cluster
                    dot_file.add_link(
                        DotLink(
                            src=DotLinkEnd(id=ref_doc, port="title"),
                            dst=DotLinkEnd(id=doc, port=dst_port, dir="w"),
                        )
                    )

        self._filedata = f"{dot_file}"

        return super().generate()
