@extern(embed)

package signed_docs

import (
	"github.com/input-output-hk/catalyst-libs/specs/cddl"
)

docs: "Contest Ballot Checkpoint": {
	description: """
		Periodically as ballots are collected, a summary of all newly collected ballots will be
		published in a `Contest Ballot Checkpoint` document.
		This document forms part of the bulletin boards complete Contest Ballot Checkpoint.

		These documents are chained to each other, and the final document is specified as final
		in the `chain` metadata.

		Typically each `Contest Ballot Checkpoint` document is made immutable by referencing it on
		the blockchain most applicable to the Contest.

		Different blockchains will have different mechanisms for referencing the individual 
		`Contest Ballot Checkpoint` documents.

		For example, Cardano will encode a `document_ref` in metadata, signed by the ballot box
		operator.

		The blockchain record must be as close in time as practically possible to the creation of
		the `Contest Ballot Checkpoint` document.
		"""
	validation: """
		* The `parameters` metadata *MUST* point to the Contest the ballot is being cast in.
		* The 'ref' metadata fields reference the Contest Ballots collected in the proceeding
			period by the ballot box.
			These are sorted from earliest `document_id`:`document_ver` regardless of the time
			the individual ballot was received by the ballot box.
		* Ballot boxes will not accept ballots whose `document_id`:`document_ver` fall outside
			the boundaries of the contest, or are not close in time to when the ballot box
			received the ballot.
		"""
	business_logic: {
		front_end: """
			* This document is not produced by the Front End.
			* The Front End may read the document to validate a given proof validates against a given
			  `smt-root` and `smt-entries`.
			"""
		back_end: """
			* Validate the ballots being referenced exist and are valid for the contest.
			* Signed by an authoritative Ballot Box.
			* All referenced ballots are in the same contest as specified in the `parameters` metadata.
			* The Chain is intact and this document is consistent with the metadata in the previous checkpoint document.
			* There is no previous checkpoint document which already references the same chained checkpoint document.
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
		chain: {
			required: "yes"
		}
	}

	headers: "content type": value: "application/cbor"

	payload: {
		description: """
			The Payload is a CBOR Document, and must conform to this schema.

			It consists of an array which defines the weights to be applied to the chosen delegations.

			Each valid delegate gets the matching weight from this array.
			The total voting power is split proportionally based on these weights over the
			valid drep nominations.
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
