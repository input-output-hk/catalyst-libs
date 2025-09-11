@extern(embed)

package signed_docs

docs: "Contest Ballot Register": {
	description: """
		Periodically as ballots are collected, a summary of all newly collected ballots will be
		published in a `Contest Ballot Register` document.
		This document forms part of the bulletin boards complete Contest Ballot Register.

		These documents are chained to each other, and the final document is specified as final
		in the `chain` metadata.

		Typically each `Contest Ballot Register` document is made immutable by referencing it on
		the blockchain most applicable to the Contest.

		Different blockchains will have different mechanisms for referencing the individual 
		`Contest Ballot Register` documents.

		For example, Cardano will encode a `document_ref` in metadata, signed by the ballot box
		operator.

		The blockchain record must be as close in time as practically possible to the creation of
		the `Contest Ballot Register` document.
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
		ref: {
			required: "yes"
			type:     "Rep Nomination"
			multiple: true
		}
		parameters: {
			required: "yes"
			type:     "Contest Parameters"
			linked_refs: [
				"ref",
			]
		}
		revocations: required: "optional"
	}

	headers: "content type": value: "application/json"

	payload: {
			description: """
				The Payload is a JSON Document, and must conform to this schema.

				It consists of an array which defines the weights to be applied to the chosen delegations.

				Each valid delegate gets the matching weight from this array.
				The total voting power is split proportionally based on these weights over the
				valid drep nominations.
				"""
			schema: _ @embed(file="payload_schemas/contest_delegation.schema.json")
			examples: [
				{
					title: "Three Delegation Weights"
					description: """
						If there are only 1 delegation, then the weights do not matter.
						If there are two, then the first delegate has a weight of 10/30, and the second has 20/30.
						If there are 5, then the weights are: `[10,20,30,1,1]`
						"""
					example: _ @embed(file="payload_schemas/contest_delegation.example.json")
				}
			]
	}

	signers: roles: user: [
		"Registered",
	]
	authors: "Neil McAuliffe": "neil.mcauliffe@iohk.io"
	versions: [
		{
			version:  "0.01"
			modified: "2025-06-19"
			changes: """
				* First Published Version
				"""
		},
		{
			version:  "0.1.2"
			modified: "2025-09-04"
			changes: """
				* Allow Multi Delegation
				"""
		},
	]
}
