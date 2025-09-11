@extern(embed)

package signed_docs

docs: "Contest Delegation": {
	description: """
		Delegation by a Registered User to a Representative for
		a contest.

		This delegation allows votes cast by the Representative
		to use the voting power of the delegating User, in addition
		to their own personal voting power and that of all other Users 
		who delegate to the same Representative.

		Delegation is for a specific Contest.
		Multiple Delegations must be published if there are multiple
		Contests within a Brand/Campaign or Category.

		This is because different Contests may have different rules.
		And not all Representatives will choose to (or be able to) nominate
		for every Contest.

		A Representative ***MAY NOT*** delegate to a different Representative
		for any contest they have nominated for.
		They ***MAY*** however nominate a Representative in any contest they
		have not nominated for.

		A Representative is NOT required to delegate to themselves in a contest they are nominated for,
		and in fact, any self-delegation is invalid and ignored.
		A Representative has an implicit 100% voting power delegation to themselves in any contest 
		they are nominated.
		The MAY not vote personally, and if they do, that vote will have Zero (0) voting power.
		100% of their voting power is assigned to their delegate vote and can not be split in any way.

		A voter MAY choose multiple delegates for a contest, in this case they are listed in priority 
		order from highest priority to lowest.
		Priority only affects two aspects of the delegation.

		1. Any residual voting power after it is split among all delegates is given to the highest 
		   priority delegate (first).
		2. If there is not enough voting power to distribute, then its distributed from highest 
		   priority to lowest.  This may mean that low priority delegates get zero voting power.
		
		An example:  If a Voter has 100 raw voting power, after quadratic scaling, they have 10.
		If they delegated to 15 delegates equally, then only the first 10 would get 1 voting power
		each.  Voting power is not fractionally assigned.

		The payload MAY contain a json document which consists of a single array which can adjust 
		the ratio of the delegation.  Voting power is divided based on the weight of a single 
		delegate over the sum of all weights of all delegates.  
		This is performed with integer division.
		As a special condition, 0 or any negative value is equivalent to a weight of 1.
		As explained above, if there is not enough voting power to distribute, low priority reps 
		will receive 0 voting power from the delegation.  And if there is any residual after integer
		division its applied to the representative with the highest priority.
		"""
	validation: """
			* The `parameters` metadata *MUST* point to the same Contest as the 
				Nomination of the Representative.
			* The 'ref' metadata field MUST point to a valid 'Representative Nomination'.
			    * IF there are multiple representatives, then any which are not pointing
				  to a valid `Representative Nomination` are excluded.  
				  The nomination is only invalid if ALL references `Representative Nomination` 
				  references are invalid.
				  This is to prevent a Representative changing their nomination invalidating a
				  delegation with multiple representatives.
			* The payload MUST be nil.

			A Representative *MUST* Delegate to their latest Nomination for a Category,
			otherwise their Nomination is invalid.

			\(docs."Rep Nomination"._latest_nomination_note)

			A Voter may withdraw their Delegation by revoking all delegation documents.
			`revocations` must be set to `true` to withdraw a delegation, OR
			a later contest delegation may change the delegated representative without
			first revoking the prior delegation, as only the latest delegation is
			considered.
			"""
	business_logic: {
		front_end: """
			* Allows a voter to select a Representative from a list of eligible candidates for a category.
			* The voter signs this document to confirm their delegation choice.
			"""
		back_end: """
			* Verifies that the voter and Representative are valid and registered for the category.
			* Records the delegation of voting power from the voter to the Representative.
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
			},
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
