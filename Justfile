# use with https://github.com/casey/just
#
# Developer convenience functions

default:
    @just --list --unsorted

# Fix and Check Markdown files
check-markdown:
    earthly +markdown-check-fix

# Check Spelling
check-spelling:
    earthly +clean-spelling-list
    earthly +check-spelling


# Pre Push Checks - intended to be run by a git pre-push hook.
pre-push: check-markdown check-spelling
    just rust/pre-push
