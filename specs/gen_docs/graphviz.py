# Graphviz DOT file generation functions.

from textwrap import indent

DEFAULT_FONT_NAME = "helvetica"
DEFAULT_FONT_SIZE = 32
DEFAULT_FONT_COLOR = "#29235c"


class DotSignedDoc:
    def __init__(self, id: str):
        self.id = id
        self.table = {
            "BORDER": 2,
            "COLOR": "#29235c",
            "CELLBORDER": 1,
            "CELLSPACING": 0,
            "CELLPADDING": 10,
        }
        self.title = {
            "PORT": "title",
            "BGCOLOR": "#1d71b8",
            "FONT COLOR": "#ffffff",
        }
        self.row_bg_colors = ["#e7e2dd"]
        self.rows = []

    def add_row(self, name: str, value: str | list[str]):
        """
        add a row of data to the table.
        """
        self.rows.append({"name": name, "value": value})

    def table_options(self) -> str:
        """
        Generate the set table options.
        """
        options = ""
        for option in self.table:
            options += f' {option}="{self.table[option]}"'
        return options

    def table_row(self, row: dict, bgcolor: str) -> str:
        value = row["value"]
        if isinstance(value, list):
            value = "<BR />".join(value)

        return f"""        <TR>
            <TD ALIGN="LEFT" PORT="{row["name"]}" BGCOLOR="{bgcolor}">
                <TABLE CELLPADDING="0" CELLSPACING="0" BORDER="0">
                    <TR>
                        <TD ALIGN="LEFT">{row["name"]} </TD>
                        <TD ALIGN="RIGHT">
                            <FONT><I>{value}</I></FONT>
                        </TD>
                    </TR>
                </TABLE>
            </TD>
        </TR>
"""

    def table_rows(self) -> str:
        """
        Generate rows for the table.
        """
        row_entries = ""
        row_count = 0
        for row in self.rows:
            row_entries += self.table_row(row, self.row_bg_colors[row_count])
            row_count += 1
            if row_count >= len(self.row_bg_colors[row_count]):
                row_count = 0
        return row_entries

    def __repr__(self) -> str:
        return "DotSignedDoc()"

    def __str__(self) -> str:
        """
        Generate the table created.
        """

        return f"""
"{self.id}" [
    id="{self.id}";
    label=<
        <TABLE{self.table_options()}>
        <TR>
            <TD PORT="{self.title["PORT"]} BGCOLOR="{self.title["BGCOLOR"]}>
                <FONT COLOR="{self.title["FONT COLOR"]}><B> {self.id} </B></FONT>
            </TD>
        </TR>
    >
    {self.table_rows()}
];
"""


class DotLink:
    def __init__(self, src_id: str, src_port: str, dst_id: str, dst_port: str | None):
        self.src_id = src_id
        self.src_port = src_port
        self.dst_id = dst_id
        self.dst_port = dst_port
        self.penwidth = 3
        self.color = "#29235c"
        self.headlabel = "1"
        self.taillabel = "*"

    def __eq__(self, other: "DotLink"):
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
        return "DotLink()"

    def __str__(self) -> str:
        link = f'"{self.src_id}":"{self.src_port}":e -> "{self.dst_id}":'
        if self.dst_port is None:
            link += '"title"'
        else:
            link += f'"{self.dst_port}"'
        return (
            f'"{link}":w [dir=forward, penwidth={self.penwidth}, color="{self.color}"'
            + f' headlabel="{self.headlabel}" taillabel="{self.taillabel}"'
        )


class DotFile:
    def __init__(self, id: str):
        self.id = id
        self.rankdir = "LR"
        self.graph = {
            "fontname": DEFAULT_FONT_NAME,
            "fontsize": DEFAULT_FONT_SIZE,
            "fontcolor": DEFAULT_FONT_COLOR,
            "bgcolor": "transparent",
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
            "fontcolor": DEFAULT_FONT_COLOR,
            "color": DEFAULT_FONT_COLOR,
        }

        self.tables = []
        self.links = []

    def add_table(self, table: DotSignedDoc):
        """
        Add a table to the graph.
        """
        self.tables.append(table)

    def __repr__(self) -> str:
        return "DotFile()"

    def __str__(self) -> str:
        """
        Generate the DOT file.
        """

        def defaults(name: str, settings: dict) -> str:
            """
            Expand the defaults.
            """
            defaults = []
            for default in settings:
                defaults.append(f'{default}="{settings[default]}"')
            return f"    {name} [{', '.join(defaults)}];"

        return f"""digraph "{self.id}" {{ 
    rankdir="{self.rankdir}"
    {defaults("graph", self.graph)}
    {defaults("node", self.node)}
    {defaults("edge", self.edge)}

{indent(self.contents, "    ")}
}}
"""
