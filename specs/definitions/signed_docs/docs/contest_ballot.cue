@extern(embed)

package signed_docs

import (
	"github.com/input-output-hk/catalyst-libs/specs/cddl"
)

docs: "Contest Ballot": {
	description: """
		An individual Ballot cast in a Contest by a registered user.

		Each ballot contains choices for all possible proposals eligible for the 
		contest.

		Multiple contest ballots can be cast by the same registered user in a contest, but
		only the latest (by its document_version) will be counted.

		The reason the ballot is cast in a contest is because there may be multiple contests in
		a campaign, and they may be attached to either the brand, campaign or category level.
		Each level, (for example category) can in-theory have multiple contests.

		Only eligible users can cast ballots in the respective contest.
		"""
	validation: """
		* The `parameters` metadata *MUST* point to the Contest the ballot is being cast in.
		* The 'ref' metadata fields within the ballot payload (not the headers) must point to 
		  ALL the proposals eligible to be chosen in the contest.
		"""
	business_logic: {
		front_end: """
			* Always cast a ballot for all proposals in the contest.
			* Any proposal not explicitely selected by a user must have the default selection applied.
			  Typically, this would be `abstain`.
			* The voter signs this document to confirm their ballot.
			* Ballots can not be cast outside the time allowed for the casting of ballots.
			* The `document_id` and `document+ver` must be within the time of allowed casting
			  of ballots.  Any document_id of document_ver outside this time are invalid and will
			  not be counted.
			"""
		back_end: """
			* Verifies that the Contest is valid, and that the ballot is cast in the appropriate 
			  time frame, and has a valid `document_id` and `document_ver` in that range.
			* Verify the payload lists all the eligible proposals which can be chosen in the contest.
			* Verify the proofs in the payload are correct.
			"""
	}

	metadata: {
		parameters: {
			required: "yes"
			type:     "Contest Parameters"
		}
		revocations: required: "optional"
	}

	headers: "content type": value: "application/cbor"

	payload: {
		description: """
			The Payload is a CBOR document that must conform to the `contest-ballot-payload` CDDL.

			Contents
			
			* `document_ref => choices`
				* The payload is a map keyed by a proposal `document_ref`.
				* Each key identifies one specific proposal via `[document_id, document_ver, document_locator]`.
				* The value for each key is that voter’s `choices` for that proposal.
				* There is exactly one set of `choices` per referenced proposal (no duplicates).

			* `choices`
				* Discriminated union of unencrypted or encrypted choices.

			* `row-proof` (optional, inside encrypted choices)
			  	* Proves, without revealing contents, that the encrypted row encodes a unit vector with exactly one selection.

			* `column-proof` (optional, top-level)
				* Placeholder for future column-level proofs across proposals.
				* Not defined at present; omit in implementations.

			* `matrix-proof` (optional, top-level)
				* Placeholder for future matrix-wide proofs across all proposals and positions.
				* Not defined at present; omit in implementations.

			* `voter-choice` (optional, top-level)
				* This is ONLY Not included when the vote is unencrypted.
				* Allows a voter to read back their ballot selections without decrypting the entire ballot.

			Notes
			
			* `document_locator` uses a CBOR Tag 42 `cid` to locate the referenced proposal in content-addressed storage.
			  Implementations should constrain the CID to SHA2-256 multihash; the multicodec SHOULD be `cbor (0x51)` to
			  reflect an unwrapped COSE_Sign CBOR block.
			* The application defines the permissible range and semantics of `clear-choice` integers.
			* All CBOR must use core-deterministic encoding so that content addressing remains stable.
			"""
		schema:   "contest-ballot-payload"
		examples: cddl.cddlDefinitions["\(schema)"].examples
	}

	signers: roles: user: [
		"Registered",
	]
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
