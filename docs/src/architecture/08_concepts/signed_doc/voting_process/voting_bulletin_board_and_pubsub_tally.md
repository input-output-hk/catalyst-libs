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

## End-to-End Voting Flow (No dReps)

1. Voter Registration
    * A user registers a Catalyst ID on-chain.
    * The default on-chain role enables them to act as a `Registered` voter in signed-docs.
2. Determine Eligible Proposals
    * After Proposal Submission and Moderation deadlines close for a contest’s anchor
      (typically Category, less commonly Campaign or Brand), the backend and frontend can
      compute the final set of eligible proposals for that contest.
    * Eligibility is derived from:
        * Proposals with `final` submission actions from all required signers by the deadline; and
        * Not disqualified or hidden by moderation, per
          [Proposals, Templates, and Discovery](./proposals_templates_and_discovery.md).
3. Voting Window and Mode
    * Contest Parameters specify when voting opens and closes and the voting mode
      (e.g., private, public, encrypted).
    * Ballots cast outside this window are invalid for tally.
4. Casting a Ballot
    * When voting opens, a registered voter may cast a [Contest Ballot](../docs/contest_ballot.md)
      for that contest.
    * The ballot payload **MUST** include a choice for every eligible proposal for that contest.
      If any eligible proposal is missing, or if extra unexpected proposals are present,
      the ballot is invalid.
5. Logging Ballots
    * The backend or pub/sub network accepts the ballot, verifies it, and logs it for the contest
      under the voter’s Catalyst ID.
6. Re-voting and Latest-Ballot Semantics
    * A voter may submit new ballots for the same contest any time until voting closes.
    * Only the latest valid ballot per (voter, contest) is counted; earlier versions are ignored.
7. Voting Power Computation
    * When voting closes, the system gathers the latest valid ballot for each registered voter
      in the contest.
    * For each voter, it computes their voting power according to the Contest Parameters
      (e.g., stake-based rules, caps, or other policies).
8. Applying Voting Power and Aggregation
    * The system applies each voter’s voting power to the selections in their ballot.
    * For each proposal, voting power from all voters with valid ballots is accumulated to produce
      per-proposal totals.
9. Bulletin Board Checkpoints
    * During voting, the bulletin board (ballot box) periodically publishes
      [Contest Ballot Checkpoints](../docs/contest_ballot_checkpoint.md) that commit to the
      collected ballots via an SMT root.
    * These checkpoints may be anchored on-chain to provide an immutable, time-stamped log of
      ballots collected up to each checkpoint.
10. Tally and Outcome
    * After voting closes and tally completes:
        * For encrypted ballots, aggregated ciphertexts are decrypted with proofs as described in
          [Voting Protocol Cryptography](./crypto.md#tally).
        * For each proposal, the total voting power applied determines the outcome and winners
          according to the Contest Parameters (e.g., thresholds, ranking, number of seats).

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
