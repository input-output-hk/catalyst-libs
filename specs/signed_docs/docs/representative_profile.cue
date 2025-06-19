@extern(embed)

package signed_docs

docs: #DocumentDefinitions & {
	"Representative Profile": {
		description: """
			## Representative Profile Document

			A Representative-specific profile, extending the minimal profile with Representative-specific fields.
			"""
		validation: """
			  - The signer MUST be a registered 'Representative'.
			  - The payload MUST be valid against the JSON schema defined in the referenced 'Representative Profile Template'.
			"""
		business_logic: {
			front_end: """
				- Display and allow editing of the Representative's core profile fields.
				- This profile serves as the central hub for a Representative's identity across all funds and categories.
				"""
			back_end: """
				- Validate Representative profile data against the referenced 'Representative_Profile_Template' and store it in the system.
				- This global profile is the foundational document referenced by all of the Representative's category-specific profiles.
				"""
		}
		metadata: {
			template: {
				required: "yes"
				type:     "Representative Profile Template"
			}
		}
		payload: {
			description: """
				The Representative profile payload contains all base profile fields and Representative-specific fields.
				Its structure is defined by the referenced Representative Profile Template.
				"""
		}
		signers: {
			roles: {
				user: [
					"Representative",
				]
			}
		}
		authors: {
			"Neil McAuliffe": "neil.mcauliffe@iohk.io"
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
}
