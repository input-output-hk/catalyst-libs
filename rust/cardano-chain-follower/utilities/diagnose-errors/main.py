"""Diagnose errors from a transaction error report."""

import argparse
import json
import operator
import os
import re
import subprocess
from typing import Any

import rich
from rich.markdown import Markdown
from rich.table import Table
from rich_argparse import ArgumentDefaultsRichHelpFormatter

description = """
# Error Report Diagnostic Analysis and Information Generator.

Takes an error report, gathers necessary on-chain data.
Process it, and outputs it in a clean way to help with diagnosing errors, and development.
"""


def get_args() -> argparse.Namespace:
    """Get the arguments of the script."""
    parser = argparse.ArgumentParser(
        description=Markdown(description, style="argparse.text"),  # type: ignore reportArgumentType
        formatter_class=ArgumentDefaultsRichHelpFormatter,
    )
    parser.add_argument(
        "--not-before-slot",
        metavar="SLOT NUMBER",
        nargs="?",
        type=int,
        default=0,
        help="Don't consider any errors in the report earlier than this slot number.",
    )

    parser.add_argument(
        "--verbose",
        type=bool,
        default=False,
        help="Show everything from the follower, not just the transactions.",
    )

    parser.add_argument(
        "--chain-follower",
        metavar="executable",
        nargs="?",
        type=str,
        default="../../target/release/examples/follow_chains",
        help="Full path and name of the chain follower executable to invoke. ()",
    )

    parser.add_argument(
        "filename",
        nargs="?",
        type=argparse.FileType("r"),
        help="Path to an error report file (One Json Error report per line)",
    )

    return parser.parse_args()


def pretty_cbor_diag(diag_str: str) -> str:
    """Make CBOR diag pretty."""

    def hex_replacer(match: re.Match[str]) -> str:
        """Convert CBOR diagnostic hex strings (e.g., h'...') to Python string representation."""
        hex_str = match.group(1).replace(" ", "")
        return f'"{hex_str}"'  # Represent bytes as hex string

    # Replace h'...' with "hexbytes"
    diag_str = re.sub(r"h'([0-9a-fA-F ]+)'", hex_replacer, diag_str)

    # Replace keys without quotes (e.g., 0:) with "0":
    diag_str = re.sub(r'(?<!")(\b\d+\b)(?=\s*:)', r'"\1"', diag_str)

    # Replace unquoted dictionary keys inside nested dictionaries
    diag_str = re.sub(r"({|,)\s*(\d+)\s*:", r'\1 "\2":', diag_str)

    # Replace single quotes with double quotes for string parsing
    diag_str = diag_str.replace("'", '"')

    # Now parse it as JSON
    try:
        obj = json.loads(diag_str)
    except json.JSONDecodeError as e:
        msg = f"Failed to parse diagnostic string: {e}"
        raise ValueError(msg) from e

    # Pretty-print
    return json.dumps(obj, indent=2)


def hex_to_bytes(hex_str: str) -> bytes:
    """Convert a byte string array to actual bytes."""
    # Convert hex strings to integers
    return bytes([int(h.strip(), 16) for h in hex_str[1:-1].split(",")])


def hex_dump(data: bytes, max_per_line: int = 65535) -> str:
    """Dump hex bytes with a line break."""
    lines: list[str] = []
    for i in range(0, len(data), max_per_line):
        chunk = data[i : i + max_per_line]
        hex_line = " ".join(f"{b:02x}" for b in chunk)
        lines.append(hex_line)
    return "\n".join(lines)


def dump_transaction(
    net: str, txn: dict[str, Any], errors: dict[str, dict[str, Any]]
) -> None:
    """Nicely dump a transaction to the screen."""
    # rich.print_json(json.dumps(txn))  # noqa: ERA001
    data = txn["fields"]
    slot = data["slot"]
    slot_time = data["slot_time"]
    txn_id = data["transaction_id"]
    txn_error: dict[str, Any] = errors[f"0x{txn_id}"]

    txn_body = hex_dump(hex_to_bytes(data["transaction_body"]))
    txn_body_diag = pretty_cbor_diag(data["transaction_body_diag"])
    txn_witness = hex_dump(hex_to_bytes(data["transaction_witness_set"]))
    txn_witness_diag = pretty_cbor_diag(data["transaction_witness_set_diag"])
    txn_aux = hex_dump(hex_to_bytes(data["transaction_aux_data"]))
    txn_aux_diag = pretty_cbor_diag(data["transaction_aux_data_diag"])
    cip509_from_chain = data["cip509"]

    cat_id = f"id.catalyst://{txn_error['catalyst_id']}"
    try:
        problem_report = json.loads(txn_error["problem_report"])
        problem_report_formatted = f"* {problem_report['context']}\n"
        for entry in problem_report["report"]:
            problem_report_formatted += f"  * {entry['context']}\n"
            problem_report_formatted += f"    * {entry['kind']['description']}\n"
    except Exception as exc:  # noqa: BLE001
        problem_report_formatted = (
            f"Failed to parse {exc}\n{txn_error['problem_report']}"
        )

    link_subdomain = "" if net == "mainnet" else f"{net}."

    rich.print(
        Markdown(
            f"""
# {net} - ***{slot_time} : {slot}*** : [Transaction {txn_id}](https://{link_subdomain}cardanoscan.io/transaction/{txn_id})

## [{cat_id}](https://input-output-hk.github.io/catalyst-libs/architecture/08_concepts/rbac_id_uri/catalyst-id-uri/)

{problem_report_formatted}

## Transaction Data Dumps

"""
        )
    )

    table = Table(title="Transaction Body")
    table.add_column("Representation", justify="right", style="cyan", no_wrap=True)
    table.add_column("Data", style="magenta")
    table.add_row("Bytes", txn_body)
    table.add_row("CBOR", txn_body_diag)
    rich.print(table)

    table = Table(title="Transaction Witness Set")
    table.add_column("Representation", justify="right", style="cyan", no_wrap=True)
    table.add_column("Data", style="magenta")
    table.add_row("Bytes", txn_witness)
    table.add_row("CBOR", txn_witness_diag)
    rich.print(table)

    table = Table(title="Transaction Auxiliary Data")
    table.add_column("Representation", justify="right", style="cyan", no_wrap=True)
    table.add_column("Data", style="magenta")
    table.add_row("Bytes", txn_aux)
    table.add_row("CBOR", txn_aux_diag)
    table.add_row("CIP509", cip509_from_chain)

    rich.print(table)


def data_thread() -> None:  # noqa: C901
    """Run the script."""
    args = get_args()

    errors: list[dict[str, Any]] = []
    for raw_error_line in args.filename:
        # Error line has escaped escaping, we need to de escape the escaped escaping.
        error_line = str(raw_error_line).replace("\\\\", "\\")
        error_data: dict[str, Any] = dict[str, Any](json.loads(error_line))  # type: ignore reportUnknownMemberType
        if error_data["slot_no"] >= args.not_before_slot:
            errors.append(error_data)
    errors.sort(key=operator.itemgetter("slot_no"))

    all_errors: dict[str, dict[str, Any]] = {}
    first_slot = errors[0]["slot_no"]
    net = str(errors[0]["catalyst_id"]).split(".")[0]
    if net not in {"preprod", "preview"}:
        net = "mainnet"

    rich.print(
        f"Searching [green]{net}[/green] "
        f"for [yellow]{len(errors)}[/yellow] transactions, "
        f"starting at slot [cyan]{first_slot}[/cyan]:\n\n"
    )

    for error in errors:
        all_errors[error["txn_id"]] = error
    params: list[str] = [
        f"{args.chain_follower}",
        f"--{net}",
        "--mithril-sync-workers=64",
        "--mithril-sync-chunk-size=16",
        "--mithril-sync-queue-ahead=6",
        f"--start-at-slot={first_slot}",
    ]
    # Get short txn ids to search for.
    params.extend([f"-t={txn_id[2:10]}" for txn_id in all_errors])

    # Start the process
    env = os.environ.copy()
    env["JSON_LOG"] = "True"
    with subprocess.Popen(  # noqa: S603
        params,
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,  # merge stderr into stdout (optional)
        text=True,  # decode bytes to str
        bufsize=1,  # line-buffered
        env=env,  # Forces output to be json so we can easily process it below.
    ) as proc:
        # Read output line by line
        for line in proc.stdout:  # type: ignore reportOptionalIterable
            try:
                json_line = json.loads(line)
                if json_line["fields"]["message"] == "Finished":
                    break  # This weird trick tries to help us finish reliably.
                if json_line["fields"]["message"] != "Transaction Dump":
                    if args.verbose:
                        rich.print_json(line, indent=None)
                    continue
                if args.verbose:  # Verbose mode, just dump everything
                    rich.print_json(line)
                    continue
                # Structured dump for easier analysis.
                dump_transaction(net, json_line, all_errors)
            except Exception:  # noqa: BLE001
                rich.print(line.rstrip())
        rich.print("Finished Finding Transactions.")
        proc.kill()


def main() -> None:
    """Run the script."""
    data_thread()


if __name__ == "__main__":
    main()
