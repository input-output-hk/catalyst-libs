"""Generate the document relationship diagram files."""

import argparse
import textwrap

from gen.doc_generator import DocGenerator
from gen.graphviz_doc_diagram import Cluster, DotFile, DotLink, DotLinkEnd, DotSignedDoc, TableRow
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
            doc_type = self.add_doc_ref_links
            cluster = self._spec.doc_in_cluster_name(doc)
            if cluster is not None:
                cluster = Cluster(name=cluster)
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
                    doc_table.add_row(TableRow(name=meta, value=doc_data.type, value_font="Courier"))
                    continue

                if doc_metadata.format == "Document Reference":
                    doc_type = doc_metadata.type
                    cluster = self._spec.ref_in_cluster(doc_type)
                    doc_type_links = doc_type
                    if cluster is not None:
                        # Clusters only need a single reference link.
                        cluster = Cluster(name=cluster.name)
                        doc_type_links = [doc_type[0]]
                    for link_dst in doc_type_links:
                        dst_dir = "n" if doc == link_dst else "w"

                        # Add dummy destination table, in case we don't have it in our docs.
                        dummy_table = DotSignedDoc(
                            table_id=link_dst,
                            title_href=Metadata.doc_ref_link(
                                link_dst,
                                depth=self._depth,
                                html=True,
                            ),
                            cluster=cluster,
                        )
                        dot_file.add_table(dummy_table)

                        dot_file.add_link(
                            DotLink(
                                src=DotLinkEnd(id=doc, port=meta),
                                dst=DotLinkEnd(id=link_dst, port="title" if cluster is None else cluster, dir=dst_dir),
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

        for doc_name in self._spec.document_names():
            if doc_name not in doc_names:
                # Add any documents as dummies with links, if they reference any docs in our doc_names.
                doc = self._spec.get_document(doc_name)
                cluster = self._spec.doc_in_cluster_name(doc_name)
                if cluster is not None:
                    print(f"{doc_name} in cluster {cluster} for {doc_names}")
                    cluster = Cluster(name=cluster)
                refs = doc.all_references()
                for ref in refs:
                    if ref in doc_names:
                        if cluster is not None:
                            print(f"{doc_name} in cluster {cluster} for {doc_names}")
                        dummy_src_table = DotSignedDoc(
                            table_id=doc_name,
                            title_href=Metadata.doc_ref_link(doc_name, depth=self._depth, html=True),
                            cluster=cluster,
                        )
                        dot_file.add_table(dummy_src_table)
                        dot_file.add_link(
                            DotLink(
                                src=DotLinkEnd(id=doc_name, port="title"),
                                dst=DotLinkEnd(id=ref, port="title" if cluster is None else cluster, dir="w"),
                            )
                        )

        self._filedata = f"{dot_file}"

        return super().generate()
