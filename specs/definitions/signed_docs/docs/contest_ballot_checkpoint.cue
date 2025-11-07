@extern(embed)

package signed_docs

import (
	"github.com/input-output-hk/catalyst-libs/specs/cddl"
)

docs: "Contest Ballot Checkpoint": {
	description: """
		Periodically, as ballots are collected, a summary of all newly collected ballots is
		published in a `Contest Ballot Checkpoint` document.
		Each checkpoint accumulates state over time,
		committing to the current set of accepted ballots via an SMT root and entry
		count,
		and optionally listing any ballots rejected in the same interval.

		Checkpoint documents are chained together. 
		The final document in the sequence is indicated by the `chain` metadata.

		Typically each `Contest Ballot Checkpoint` is made immutable by referencing it on 
		the	blockchain most applicable to the Contest.

		Different blockchains will have different mechanisms for referencing checkpoint
		documents.
		For example, Cardano can encode a `document_ref` in on‑chain metadata,
		signed by the ballot‑box (bulletin board) operator.

		The blockchain record should be as close in time as practically possible to the
		creation of the `Contest Ballot Checkpoint` document to provide a reliable anchor for
		proofs of inclusion and auditability.
		"""
	validation: """
		* `parameters` metadata MUST reference the Contest this checkpoint pertains to.
		* `ref` metadata MUST reference the accepted Contest Ballots collected in the preceding
		  interval by the bulletin board.
		  Entries MUST be sorted by ascending `document_id`:`document_ver`,
		  regardless of the arrival time at the bulletin board.
		* Ballot boxes MUST reject ballots whose `document_id`:`document_ver` fall outside the
		  contest’s allowed time window,
		  or that are not close in time to when the ballot box received the ballot.
		* When present, `rejections` MUST only contain recognized reasons and valid
		  `document_ref` values of Contest Ballot documents;
		  rejected ballots MUST NOT appear in `ref` for the same interval.
		* `smt-root` MUST be the Blake3 root hash of the canonical SMT containing all accepted
		  ballots up to and including this checkpoint;
		* `smt-entries` MUST equal the total count of leaves in that SMT.
		* `chain` MUST be intact and consistent: 
		  the previous checkpoint referenced by `chain`
		  MUST exist, match type, id, and parameters, and have a lower `ver` and height exactly
		  one less than this checkpoint.
		"""
	business_logic: {
		front_end: """
			* Not produced by the Front End.
			* May be read to verify that a proof of inclusion validates against the published
			  `smt-root` and `smt-entries`.
			"""
		back_end: """
			* Validate that all referenced ballots exist and are valid for the contest.
			* Ensure the document is signed by an authoritative bulletin‑board operator.
			* Ensure all referenced ballots are for the same contest as `parameters`.
			* Compute and verify `smt-root` and `smt-entries` against the current SMT state.
			* If present, validate `rejections` reasons and that rejected `document_ref`s are
			  Contest Ballot documents.
			* Ensure the chain is intact and consistent with the previous checkpoint.
			* Ensure no previous checkpoint already chains to the same target (no forks within a
			  single authoritative sequence).
			"""
	}

	metadata: {
		ref: {
			required: "yes"
			type:     "Contest Ballot"
			multiple: true
		}
		parameters: {
			required: "yes"
			type:     "Contest Parameters"
			linked_refs: [
				"ref",
			]
		}
		chain: required: "yes"
	}

	headers: "content type": value: "application/cbor"

	payload: {
		description: """
			The Payload is a CBOR document that MUST conform to the
			`contest-ballot-checkpoint` CDDL schema.

			Contents

			* `stage` (required)
				* Processing stage represented by this checkpoint.
				* One of: `"bulletin-board" | "tally" | "audit"`.

			* `smt-root` (required)
				* Blake3 256‑bit digest of the root of the Sparse Merkle Tree (SMT)
				  containing all accepted ballot `document_ref`s up to and including
				  this checkpoint.

			* `smt-entries` (required)
				* The total number of documents (leaves) in the SMT at this checkpoint.

			* `rejections` (optional)
				* Map of `rejection-reason => [ document_ref, ... ]` listing ballots
				  rejected during this checkpoint interval.
				* Reasons are limited to: `"already-voted"`, `"obsolete-vote"`.

			* `encrypted-tally` (optional)
				* Placeholder map of `document_ref => encrypted-tally-proposal-result`.
				* May appear at later stages to commit to encrypted tally snapshots.

			* `tally` (optional)
				* Placeholder map of `document_ref => tally-proposal-result` for clear tally
				  snapshots.

			* `drep-encryption-key` (optional)
				* Placeholder for a DRep encryption key to allow decryption where required
				  for audit or published results.

			Notes

			* The document `ref` metadata lists the accepted Contest Ballots collected during
			  the interval covered by this checkpoint;
			  rejected ballots are listed under `rejections` and are not included in `ref` for that interval.
			* The SMT is cumulative across the chain; each checkpoint’s `smt-root` and
			  `smt-entries` commit to all accepted ballots up to that point.
			"""
		schema:   "contest-ballot-checkpoint"
		examples: cddl.cddlDefinitions["\(schema)"].examples
	}

	signers: roles: {
		user: []
		admin: [
			"Bulletin Board Operator",
		]
	}

	authors: "Steven Johnson": "steven.johnson@iohk.io"
	versions: [
		{
			version:  "0.1.5"
			modified: "2025-11-03"
			changes: """
				* Add Voting Ballots and Ballot Checkpoint Documents
				"""
		},
	]
}
