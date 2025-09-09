@extern(embed)

package signed_docs

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

	headers: "content type": value: "application/cbor"

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
