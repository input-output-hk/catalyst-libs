"""Graphviz DOT file generation functions."""

from textwrap import indent

from pydantic import BaseModel, Field

from spec.metadata import Metadata

DEFAULT_FONT_NAME = "helvetica"
DEFAULT_FONT_SIZE = 32
DEFAULT_FONT_COLOR = "#29235c"


class TableRow(BaseModel):
    """Table Row Data."""

    name: str
    value: str | list[str]
    link: str | None = Field(default=None)
    tooltip: str | None = Field(default=None)
    name_font: str | None = Field(default=None)
    value_font: str | None = Field(default=None)

    def generate(self, bgcolor: str) -> str:
        """Generate a single row of the table."""
        value = self.value
        if isinstance(self.value, list):
            value = "<BR />".join(self.value)

        link = "" if self.link is None else f' HREF="{self.link}"'
        tooltip = "" if self.tooltip is None else f' TITLE="{self.tooltip}"'
        value_font = "" if self.value_font is None else f' FACE="{self.value_font}"'
        name_font = "" if self.name_font is None else f' FACE="{self.name_font}"'

        return f"""        <TR>
            <TD ALIGN="LEFT" PORT="{self.name}" BGCOLOR="{bgcolor}"{link}{tooltip}>
                <TABLE CELLPADDING="0" CELLSPACING="0" BORDER="0">
                    <TR>
                        <TD ALIGN="LEFT" VALIGN="TOP" WIDTH="200"><FONT{name_font}>{self.name}</FONT></TD>
                        <TD ALIGN="RIGHT"><FONT{value_font}><I>{value}</I></FONT></TD>
                    </TR>
                </TABLE>
            </TD>
        </TR>
"""


def default_row_bg_color() -> list[str]:
    """Get Default Row BG Color."""
    return ["#e7e2dd", "#b8b5b0"]


class TableTheme(BaseModel):
    """Theme of a Table."""

    border: int = Field(default=2)
    color: str = Field(default="#29235c")
    cell_border: int = Field(default=1)
    cell_spacing: int = Field(default=0)
    cell_padding: int = Field(default=10)
    title_bgcolor: str = Field(default="#1d71b8")
    title_color: str = Field(default="#ffffff")
    row_bg_color: list[str] = Field(default_factory=default_row_bg_color)
    row_bg_color_offset: int = Field(default=0)

    def table(self) -> str:
        """Generate the set table options."""
        return (
            " "
            f'BORDER="{self.border}" '
            f'COLOR="{self.color}" '
            f'CELLBORDER="{self.cell_border}" '
            f'CELLSPACING="{self.cell_spacing}" '
            f'CELLPADDING="{self.cell_padding}"'
        )

    def title(self) -> str:
        """Generate the set title options."""
        return f' BGCOLOR="{self.title_bgcolor}"'

    def title_font(self) -> str:
        """Generate the set title font options."""
        return f' COLOR="{self.title_color}"'

    def next_row_bg_color(self) -> str:
        """Get next row background color."""
        next_bg = self.row_bg_color[self.row_bg_color_offset]
        self.row_bg_color_offset += 1
        if self.row_bg_color_offset >= len(self.row_bg_color):
            self.row_bg_color_offset = 0
        return next_bg


class DotSignedDoc(BaseModel):
    """Table representing a single signed document."""

    table_id: str
    title_port: str = Field(default="title")
    title_href: str | None = None
    theme: TableTheme = Field(default_factory=TableTheme)
    rows: list[TableRow] = Field(default_factory=list)

    def add_row(self, row: TableRow) -> None:
        """Add a row of data to the table."""
        self.rows.append(row)

    def has_rows(self) -> bool:
        """Are any rows defined."""
        return len(self.rows) != 0

    def table_rows(self) -> str:
        """Generate all rows for the table."""
        row_entries = ""
        for row in self.rows:
            row_entries += row.generate(self.theme.next_row_bg_color())
        return row_entries

    def __repr__(self) -> str:
        """Repr."""
        return "DotSignedDoc()"

    def __str__(self) -> str:
        """Generate the table created."""
        title_options = f'PORT="{self.title_port}"{self.theme.title()}'
        title_value = f"<FONT{self.theme.title_font()}><B> {self.table_id} </B></FONT>"
        title_href = "" if self.title_href is None else f'HREF="{self.title_href}"'
        return f"""
"{self.table_id}" [
    id="{self.table_id}";
    label=<
        <TABLE{self.theme.table()}>
        <TR>
            <TD {title_options} {title_href}>
                {title_value}
            </TD>
        </TR>
{self.table_rows()}
        </TABLE>
    >
];
"""


class DotLinkTheme(BaseModel):
    """Theme for a Link."""

    penwidth: int = Field(default=6)
    color: str = Field(default="#29235c")
    headlabel: str | None = Field(default="1")
    taillabel: str | None = Field(default="*")
    direction: str = Field(default="forward")

    def __str__(self) -> str:
        """Str."""
        options: list[str] = [
            f"dir={self.direction}",
            f"penwidth={self.penwidth}",
            f'color="{self.color}"',
        ]
        if self.headlabel is not None:
            options.append(f'headlabel="{self.headlabel}"')
        if self.taillabel is not None:
            options.append(f'taillabel="{self.taillabel}"')

        return f" [{', '.join(options)}]"


class DotLink(BaseModel):
    """Represents a Link between documents."""

    src_id: str
    src_port: str | None = Field(default=None)
    src_dir: str | None = Field(default="e")
    dst_id: str
    dst_port: str | None = Field(default=None)
    dst_dir: str | None = Field(default="w")
    theme: DotLinkTheme = Field(default_factory=DotLinkTheme)

    def __eq__(self, other: "DotLink") -> bool:
        """Eq."""
        if not isinstance(other, DotLink):
            # don't attempt to compare against unrelated types
            return NotImplemented

        return (
            self.src_id == other.src_id
            and self.src_port == other.src_port
            and self.dst_id == other.dst_id
            and self.dst_port == other.dst_port
        )

    def __repr__(self) -> str:
        """Repr."""
        return "DotLink()"

    @staticmethod
    def mk_link_name(name: str, port: str | None, direction: str | None) -> str:
        """Make a graphviz link name."""
        name = f'"{name}"'
        if port is not None:
            name += f':"{port}"'
            if direction is not None:
                name += f":{direction}"
        return name

    def src(self) -> str:
        """Return the source."""
        return self.mk_link_name(self.src_id, self.src_port, self.src_dir)

    def dst(self) -> str:
        """Return the destination."""
        return self.mk_link_name(self.dst_id, self.dst_port, self.dst_dir)

    def __str__(self) -> str:
        """Str."""
        return f"{self.src()} -> {self.dst()}{self.theme}"


class DotFile:
    """Represents and Generates a Signed Document Relationship DOT File."""

    def __init__(self, file_id: str, title: str, depth: int = 0, title_size: int = 150) -> None:
        """Init."""
        self.id = file_id
        self.title = title
        self.title_size = title_size
        self.rankdir = "LR"
        self.graph = {
            "fontname": DEFAULT_FONT_NAME,
            "fontsize": DEFAULT_FONT_SIZE,
            "fontcolor": DEFAULT_FONT_COLOR,
            "bgcolor": "white",
        }
        self.node = {
            "penwidth": 0,
            "margin": 0,
            "fontname": DEFAULT_FONT_NAME,
            "fontsize": DEFAULT_FONT_SIZE,
            "fontcolor": DEFAULT_FONT_COLOR,
        }
        self.edge = {
            "fontname": DEFAULT_FONT_NAME,
            "fontsize": DEFAULT_FONT_SIZE,
            "fontcolor": "red",
            "color": DEFAULT_FONT_COLOR,
        }
        self.depth = depth

        self.tables = {}
        self.links = []

    def add_table(self, table: DotSignedDoc) -> None:
        """Add a table to the graph.

        Will always add a table if it doesn't already exist.
        Only replace existing tables if the new table has rows.
        """
        if table.table_id not in self.tables or table.has_rows():
            self.tables[table.table_id] = table

    def add_link(self, link: DotLink) -> None:
        """Add a link to the graph.

        Will add an empty Table if the destination port is None and
        destination does not exist.
        Src is assumed to always exist.
        """
        # Add a dummy table, so the link has something to anchor on.
        # Won't add anything if it exists, and will get replaced
        # if the real table gets added later (has any rows).
        dummy_dst_table = DotSignedDoc(
            table_id=link.dst_id, title_href=Metadata.doc_ref_link(link.dst_id, depth=self.depth, html=True)
        )
        self.add_table(dummy_dst_table)  # Wont add if already exists.
        self.links.append(link)

    def __repr__(self) -> str:
        """Repr."""
        return "DotFile()"

    def __str__(self) -> str:
        """Generate the DOT file."""

        def defaults(name: str, settings: dict) -> str:
            """Expand the defaults."""
            defaults = []
            for default, value in settings.items():
                defaults.append(f'{default}="{value}"')
            return f"    {name} [{', '.join(defaults)}];"

        return f"""digraph "{self.id}" {{
    rankdir="{self.rankdir}"
    {defaults("graph", self.graph)}
    {defaults("node", self.node)}
    {defaults("edge", self.edge)}

    labelloc="t"
    label="{self.title}"
    fontcolor="#1d71b8"
    fontsize={self.title_size}

{indent("\n".join(map(str, self.tables.values())), "    ")}

{indent("\n".join(map(str, self.links)), "    ")}
}}
"""
