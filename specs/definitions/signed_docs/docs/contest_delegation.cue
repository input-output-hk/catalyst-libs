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
		And not all Representatives will choose to nominate
		for every Contest.

		A Representative ***MAY NOT*** delegate to a different Representative
		for any contest they have nominated for.
		They ***MAY*** however nominate a Representative in any contest they
		have not nominated for.
		"""
	validation: """
			* The `parameters` metadata *MUST* point to the same Contest as the 
				Nomination of the Representative.
			* The 'ref' metadata field MUST point to a valid 'Representative Nomination'.
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

	//headers: "content type": value: "application/cbor"
	metadata: {
		ref: {
			required: "yes"
			type:     "Rep Nomination"
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
	payload: {
		description: """
			  There is no payload.
			"""

		required: "excluded"
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
	]
}
