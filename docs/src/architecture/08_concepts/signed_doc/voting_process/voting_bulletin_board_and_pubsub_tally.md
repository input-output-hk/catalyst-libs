---
Title: Voting, Bulletin Board, and Pub/Sub Tally
Authors:
    - Steven Johnson <steven.johnson@iohk.io>
Created: 2025-11-05
order: 5
---

<!-- cspell: words SMT merklized -->

## Abstract

Describes the end-to-end voting flow, the bulletin board (ballot box) checkpointing model,
and how ballots and checkpoints are discovered, validated, and tallied in a decentralized pub/sub network.

## Ballots

* Document: [Contest Ballot](../docs/contest_ballot.md) — one ballot per registered user per contest
  is counted.  (latest [`ver`](../metadata.md#ver) counts, others ignored).
* Payload: [CBOR][RFC8949], contains `document_ref => choices` for all eligible proposals,
  with encrypted or clear choices and ZK proofs when encrypted.
    * See cryptography: [Voting Protocol Cryptography Schema](./crypto.md).
* Anchoring: [`metadata.parameters`](../metadata.md#parameters) points to the contest;
  ballots cast outside the configured time window are invalid.

### Validation (Consumer)

* Verify signature (`Registered`), `type`, [`id`](../metadata.md#id), [`ver`](../metadata.md#ver), `content type`.
* Verify [`metadata.parameters`](../metadata.md#parameters) matches the intended contest.
* Validate payload against [CDDL][RFC8610], including per-proposal references and proof structure.
* Ensure the payload includes exactly one set of choices per eligible proposal;
  for unselected proposals apply the default choice (typically abstain) as required by UI rules.

## Bulletin Board (Ballot Box) and Checkpoints

* Document: [Contest Ballot Checkpoint](../docs/contest_ballot_checkpoint.md).
* Role: produced periodically by the bulletin board operator (admin role: “Bulletin Board Operator”).
* Purpose: commits a merklized summary (Sparse Merkle Tree, SMT) of the ballots collected since the
  previous checkpoint.
    * Payload contains `smt-root` (BLAKE3-256 digest) and `smt-entries` (count).
    * Checkpoints are chained via `metadata.chain`; the final checkpoint height is negated.
    * Typically anchored to a blockchain (e.g., encoded `document_ref` in on-chain metadata signed
      by the operator) to provide an immutable timestamped anchor.

### Validation (Consumer)

* Verify signature (admin role), `type`, [`id`](../metadata.md#id), [`ver`](../metadata.md#ver),
  [`parameters`](../metadata.md#parameters) (contest), and [`parameters`](../metadata.md#chain) integrity.
* Confirm all referenced ballots exist and are valid for the contest and within time bounds.
* Confirm the chain has no forks and that heights are consecutive and finality is respected once a
  final height is observed.
* The SMT root enables third parties to verify inclusion proofs for ballots served by the bulletin board.

## Pub/Sub Discovery Model

Suggested topics keyed by contest:

* Ballots: `signed-docs/contest-ballot/<contest-id>`
* Checkpoints: `signed-docs/contest-ballot-checkpoint/<contest-id>`

Where `<contest-id>` is the Contest Parameters [`id`](../metadata.md#id) for the contest via
[`metadata.parameters`](../metadata.md#parameters).

### Consumer Pipeline

1. Ballot intake
   * Verify and store the latest valid ballot per (voter, contest).
2. Checkpoint intake
   * Verify and store chain; update the current finalized SMT root and entry count.
3. Optional inclusion proof verification
   * For any ballot, request an SMT path from the bulletin board and verify against the latest
     (or finalized) checkpoint root.

## Tally

Tally computes per-proposal totals from valid ballots and delegated voting power.

* Cryptographic tally for encrypted choices follows the homomorphic aggregation and
  ZK verification described in [Voting Protocol Cryptography](./crypto.md#tally).
* Delegations: incorporate the effective delegation set from
  [dRep Delegation](./drep_delegation_and_discovery.md) by adding each delegator’s voting power to
  their delegate’s ballot according to weights and priorities for the contest.
* Direct vs Delegated: systems commonly treat a voter’s direct ballot in a contest as taking precedence
  over their delegation for that contest;
  for categories not directly voted on, the delegation applies.
  Implementations should follow the contest’s configured rules.

### Tally Steps (High-Level)

* Collect the latest valid ballot per (voter, contest), including representative ballots.
* For encrypted ballots, compute option-wise homomorphic aggregates; decrypt with proofs per the spec.
* Resolve delegations to representatives with the latest valid nominations;
  distribute voting power by weights and priority.
* For encrypted ballots, compute option-wise homomorphic aggregates; decrypt with proofs per the spec.
* Publish decrypted results with tally proofs where applicable.

## Content Addressing and Retrieval

* All ballots and checkpoints include locators with a [CBOR][RFC8949] Tag 42 CID.
  See: [Document Reference](../metadata.md#document-reference).
* Pub/sub disseminates signed documents; content-addressed storage provides retrieval by CID for auditability.

## Operational Notes

* Index by (contest, voter) for ballots and (contest, height) for checkpoints.
* Enforce time windows derived from the contest parameters;
  reject or ignore ballots outside allowed windows.
* Prefer latest valid [`ver`](../metadata.md#ver) per ballot [`id`](../metadata.md#id),
  and respect checkpoint finality for inclusion proofs.

[RFC8949]: https://www.rfc-editor.org/rfc/rfc8949.html
[RFC8610]: https://www.rfc-editor.org/rfc/rfc8610
