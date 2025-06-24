package signed_docs

// Proposal Document Definition

docs: "Rep Nomination": {
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
		* Other rules may apply as defined by the Contest or other parameters which can
			control who may validly nominate as a representative in a Contest.
		"""

	business_logic: {

		front_end: """
			* Allows a Representative to create or update their profile for a category.
			* The Representative sets their status to 'active' to be discoverable for delegation.
			* The Representative can set their status to 'revoked' to signal they are no longer participating in the category, without having to revoke the document.
			"""

		back_end: """
			* The backend MUST verify the signer is a 'Representative' and that all referenced documents exist.
			* The system will only consider Representatives with an 'active' status as eligible for delegation.
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

	signers: roles: user: [
		"Representative",
	]

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
	]
}
