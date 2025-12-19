package signed_docs

// Proposal Document Definition

docs: "Rep Nomination": {
	_latest_nomination_note: """
		This is because Delegation points to a *SPECIFIC* Nomination, and it
		*MUST* be the latest for the Representative on the Contest.
		As the Nomination contains information that the User relies on
		when choosing to delegate, changing that information could have a 
		real and detrimental result in the Delegation choice.
		Therefore, for a Delegation to be valid, it *MUST* point to the
		latest Nomination for a Representative.

		Publishing a newer version of the Nomination Document to a specific contest will
		invalidate all pre-existing delegations, and all voters will need
		to re-delegate to affirm the delegates latest nomination.
		"""

	description: """
		A Representative Nomination Document is created to opt
		in as a Representative Voter for a specific Contest on a Brand/Campaign or Category.

		This Document is a kind of `Profile` that is primarily used to
		help justify the Representatives Nomination to prospective delegators.

		The user must have registered as a Representative.
		The presence of this document signifies the user's intent to participate in that 
		contest as a Representative.
		
		The document's structure is defined by the associated 
		Rep Nomination Form Template.
		This allows an Admin to specify contest-specific requirements.

		The Representative can retract their nomination by using the `revoke` metadata to
		revoke this Nomination document.

		It is an extension of all other profiles attached to the same Catalyst ID.

		Profiles themselves are intentionally general, however they may be
		linked to a Brand/Campaign/Category via the template used by the profile.

		The payload of a profile is controlled by its template.
		"""

	validation: """
		* The signer MUST be a registered 'Representative'.
		* The 'ref' metadata field MUST point to a valid 'Representative Profile' document.
		* The 'parameters' metadata field MUST point to a valid 'Contest Parameters' document.
		* The 'template' metadata field MUST point to a valid 'Representative Nomination Form Template' document.
		* The payload MUST be valid against the JSON schema defined in the referenced template.
		* Only **ONE** major version (same `id`) could be submitted per contest.
			If representative already submitted nomination for the specific contest,
			only sub-versions could be submitted by that representative
			(same `id` different `ver`).
		* Other rules may apply as defined by the Contest or other parameters which can
			control who may validly nominate as a representative in a Contest.

		No Nomination is valid unless the latest Contest Delegation of the Delegate
		refers to their own Nomination.
		This requires that Nominating is a two step process:

		1. Post the Nomination Document.
		2. Post a Contest Delegation delegating to the new Nomination Document.
		
		Updating the Nomination Document will invalidate all Nominations to the 
		Representative.

		\(_latest_nomination_note)
		"""

	business_logic: {

		front_end: """
			* Allows a Representative to create or update their profile for a category.
			* The Representative sets their status to 'active' to be discoverable for delegation.
			* The Representative `revokes` the Nomination to signal they are no longer 
			  participating in the category.
			* Nominations are not valid if the latest Delegation by the Representative does NOT
			  reference their latest Nomination.
			"""

		back_end: """
			* The backend MUST verify the signer is a 'Representative' and that all referenced documents exist.
			* Only **ONE** major version (same `id`) could be submitted per 'Representative'.
			* The system will only consider Representatives as having valid Nominations if:
				* Their latest Nomination in a Contest is not Revoked.
				* Their latest Delegation in a Contest references their latest Nomination.
			"""
	}
	headers: "content type": value: "application/json"

	metadata: {
		ref: {
			required: "yes"
			type:     "Rep Profile"
		}

		template: {
			required: "yes"
			type:     "Rep Nomination Form Template"
		}

		revocations: required: "optional"

		parameters: _metadataFieldContestParameters
		parameters: linked_refs: [
			"template",
		]
	}

	payload: description: """
		The Representative's profile data for a specific contest.
		Its structure is defined by the referenced template document.

		In the case of Revoking a nomination the payload is `nil`.
		"""

	signers: {
		roles: user: [
			"Representative",
		]

		update: type: "ref"
	}

	authors: {
		"Neil McAuliffe": "neil.mcauliffe@iohk.io"
		"Steven Johnson": "steven.johnson@iohk.io"
	}

	versions: [
		{
			version:  "0.01"
			modified: "2025-06-19"
			changes: """
				* First Published Version
				"""
		},
		{
			version:  "0.2.2"
			modified: "2025-12-02"
			changes: """
				* Added missing `signers: update: type: "ref"` definition.
				"""
		},
	]
}
