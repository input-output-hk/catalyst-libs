VERSION 0.8

IMPORT github.com/input-output-hk/catalyst-ci/earthly/mdlint:rust/bump-compiler-to-1.83 AS mdlint-ci
IMPORT github.com/input-output-hk/catalyst-ci/earthly/cspell:rust/bump-compiler-to-1.83 AS cspell-ci


FROM debian:stable-slim

# check-markdown : markdown check using catalyst-ci.
check-markdown:
    DO mdlint-ci+CHECK

# markdown-check-fix : markdown check and fix using catalyst-ci.
markdown-check-fix:
    LOCALLY

    DO mdlint-ci+MDLINT_LOCALLY --src=$(echo ${PWD}) --fix=--fix

# clean-spelling-list : Make sure the project dictionary is properly sorted.
clean-spelling-list:
    DO cspell-ci+CLEAN
        
# check-spelling : Check spelling in this repo inside a container.
check-spelling:
    DO cspell-ci+CHECK

# spell-list-words : List words in a dictionary
spell-list-words:
    FROM ghcr.io/streetsidesoftware/cspell:8.0.0
    WORKDIR /work

    COPY . .

    RUN cspell-cli --words-only --unique "wasm/**" | sort -f

# repo-docs : target to store the documentation from the root of the repo.
repo-docs:
    # Create artifacts of extra files we embed inside the documentation when its built.
    FROM scratch

    WORKDIR /repo
    COPY --dir *.md LICENSE-APACHE LICENSE-MIT .

    SAVE ARTIFACT /repo repo
