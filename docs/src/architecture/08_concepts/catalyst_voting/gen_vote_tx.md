# General Voting Transaction

---

Title: General Voting Transaction Structure

Status: Proposed

Authors:
    - Alex Pozhylenkov <alex.pozhylenkov@iohk.io>

Created: 2024-09-04

---

## Abstract

This document defines a generalized voting transaction [CDDL] structure.

## Motivation

Project "Catalyst" requires a structure to keep people vote's data in the secure and verifiable way.

## Specification

<!-- markdownlint-disable max-one-sentence-per-line code-block-style -->
??? note "vote transaction definition: `gen_vote_tx.cddl`"

    ```CDDL
    {{ include_file('./cddl/gen_vote_tx.cddl', indent=4) }}
    ```
<!-- markdownlint-enable max-one-sentence-per-line code-block-style -->

Vote:

* `choices` - a collection of voter choices for the proposal.
* `proof` - a voter proof, could be `null`.
* `prop-id` - a proposal id for which `choice` is made, could be `null`.
  For that case where for the `event` defined only **one** proposal,
  so it's redundant to provide an additional identifier for the proposal,
  so it could be placed `null`.

`voter-data` - an any additional voter's specific data.

## Rationale

## Path to Active

### Acceptance Criteria
<!-- Describes what are the acceptance criteria whereby a proposal becomes 'Active' -->

### Implementation Plan
<!-- A plan to meet those criteria or `N/A` if an implementation plan is not applicable. -->

<!-- OPTIONAL SECTIONS: see CIP-0001 > Document > Structure table -->

[CDDL]: https://datatracker.ietf.org/doc/html/rfc8610
