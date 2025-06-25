package signed_docs

// Proposal Document Definition

docs: "Rep Profile": {
	description: """
		A Rep Profile allows a representative voter to publish information
		about themselves to help explain who they are and why someone should
		consider delegating to them.

		It is an extension of all other profiles attached to the same Catalyst ID.

		Profiles themselves are intentionally general, however they may be
		linked to a brand via the template used by the profile.

		The payload of a profile is controlled by its template.
		"""

	validation: """
		* The signer MUST be a registered 'Representative'.
		* The payload MUST be valid against the JSON schema defined in the referenced 
		'Rep Profile Template'.
		"""

	business_logic: {

		front_end: """
			* Display and allow editing of the Representative's core profile fields.
			* This profile serves as the central hub for a Representative's public identity.
			"""

		back_end: """
			* Validate Representative profile data against the referenced 'Rep Profile Template' and store/index it.
			* This global profile is the foundational document referenced by all of the Rep's contest specific profiles.
			"""
	}
	headers: "content type": value: "application/json"

	metadata: {
		template: {
			required: "yes"
			type:     "Rep Profile Form Template"
		}

		revocations: required: "optional"

		parameters: _metadataFieldBrandParameters
		parameters: linked_refs: [
			"template",
		]
	}

	payload: description: """
		The Representative profile payload contains all Representative-specific fields.
		Its structure is defined by the referenced Rep Profile Template.

		Must be valid according to the schema contained within the 
		`Document Reference` from the `template` metadata.
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
