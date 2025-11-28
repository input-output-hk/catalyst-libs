# AGENTS – Guidelines for Automated Assistants

This file provides repo‑wide guidance for automated coding assistants working in
`catalyst-libs`.
It applies to the entire repository unless a more specific
`AGENTS.md` exists in a subdirectory.

## General Principles

* Make **small, targeted changes** that directly address the user’s request.
* Prefer **clarity and correctness** over cleverness; do not refactor broadly
  unless explicitly asked.
* Keep existing project structure, naming, and file layout intact unless there
  is a strong reason to change it.
* When in doubt about intent (spec vs implementation vs docs), **ask the user**
  rather than guessing.

## Documentation & Specs

* For architecture/spec docs under `docs/src/architecture` that have a matching
  Jinja source under `specs/generators/pages`, treat the **Jinja template as the
  source of truth**.
    * Example: if both
    `specs/generators/pages/signed_doc/voting_process/foo.md.jinja` and
    `docs/src/architecture/08_concepts/signed_doc/voting_process/foo.md`
    exist, edit the `.md.jinja` file, not the generated `.md`.
* Follow existing Markdown style:
    * Respect the “one sentence per line” convention where it is used.
    * Keep headings, link styles, and admonitions consistent with nearby text.
* When adding terminology that is likely to trigger the spell checker, add it
  to `.config/dictionaries/project.dic` in **sorted order**.
* If the user asks for validation, suggest or run (with their consent):
    * `just check-markdown`
    * `just check-spelling`

## Python (spec generators and tooling)

* Python code lives primarily under `specs/generators` and is linted/formatted
  with `ruff` (see `ruff.toml`).
* Conform to the existing style:
    * Use double quotes for strings.
    * Keep line length within the configured limit.
    * Do not introduce new global `ignore` rules in `ruff.toml` unless the user
      requests it.
* Where sensible, keep functions small and focused; avoid introducing new
  dependencies without discussing with the user.
* When the user wants checks run, prefer:
    * `just format-python-code`
    * `just lint-python`

## Rust Workspace

* The Rust workspace is in `rust/`.
  Follow the existing module layout and
  crate boundaries; do not split or merge crates without explicit direction.
* Match the existing style and patterns:
    * Use `cargo fmt` / the configured `fmtfix` and lint commands indirectly
      via `rust/Justfile` where possible.
    * Avoid adding new crates to the workspace unless clearly justified.
* For validation, and only when the user is ready for longer commands, suggest
  or run:
    * `cd rust && just code-format`
    * `cd rust && just code-lint`
    * `cd rust && just pre-push` (heavier, CI‑like checks)

## CUE/CDDL Specs and Generated JSON

* Specification source is primarily in `specs/definitions` (CUE, CDDL, etc.).
  Treat these as **normative** and make changes carefully and incrementally.
* Where both a CUE definition and a JSON spec exist (e.g. `specs/signed_doc.json`),
  assume the JSON is **generated** from the CUE/source definitions.
    * Prefer editing the source definitions and let the project’s existing
      tooling regenerate derived artifacts.
    * Do not hand‑edit large generated JSON files unless explicitly instructed.

## Earthly, Just, and CI

* This repo uses Earthly (`Earthfile`s) and `just` for repeatable workflows.
* Respect the intended execution context:
    * Targets or functions marked as `LOCALLY` are meant to run on the host, not
      inside container builds.
    * Do not try to retrofit such targets into containerized steps without a
      clear reason and user confirmation.
* When adding new Earthly targets or Just recipes, mirror the style of
  existing ones and keep names short and descriptive.

## Things to Avoid

* Do not:
    * Change licensing files, `CODE_OF_CONDUCT.md`, or security policies.
    * Introduce breaking API changes (in Rust or Python) without calling that
      out explicitly to the user.
    * Mass‑reformat the entire repo; limit formatting to files you touch.
* Avoid speculative “cleanup” in areas unrelated to the user’s request, even
  if you notice possible improvements; mention them in your summary instead.
