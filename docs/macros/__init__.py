from .include import inc_file
from .signed_docs import (
    cose_header_parameters,
    doc_type_details,
    doc_type_summary,
    external_links,
    signed_doc_details,
)


def define_env(env):
    """
    This is the hook for defining variables, macros and filters
    """

    @env.macro
    def include_file(filename, start_line=0, end_line=None, indent=None):
        # Provided by the base mkdocs config.
        return inc_file(env, filename, start_line, end_line, indent)

    @env.macro
    def insert_doc_type_summary():
        try:
            return doc_type_summary(env)
        except Exception as exc:
            return f"Macro Expansion Error: {exc}"

    @env.macro
    def insert_doc_type_details():
        try:
            return doc_type_details(env)
        except Exception as exc:
            return f"Macro Expansion Error: {exc}"

    @env.macro
    def insert_cose_header_parameters():
        try:
            return cose_header_parameters(env)
        except Exception as exc:
            return f"Macro Expansion Error: {exc}"

    @env.macro
    def insert_signed_doc_details(name):
        try:
            return signed_doc_details(env, name)
        except Exception as exc:
            return f"Macro Expansion Error: {exc}"

    @env.macro
    def insert_external_links():
        try:
            return external_links(env)
        except Exception as exc:
            return f"Macro Expansion Error: {exc}"
