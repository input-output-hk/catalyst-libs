from .include import inc_file
from .signed_docs import doc_type_details, doc_type_summary, signed_doc_details


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
        return doc_type_summary(env)

    @env.macro
    def insert_doc_type_details():
        return doc_type_details(env)

    @env.macro
    def insert_signed_doc_details(name):
        return signed_doc_details(env, name)
