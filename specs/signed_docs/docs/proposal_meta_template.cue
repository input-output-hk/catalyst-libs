package signed_docs

// Proposal Document Definition

docs: #DocumentDefinitions & {
	"Proposal Meta Template": {
		description: """
			## Proposal Meta Template Document
			
			A Proposal Meta Template is used to enforce functional requirements
			are met in any Proposal Template.

			The payload of a proposal template is controlled by its meta template.
			"""

		headers: {
			"content type": {
				value: "application/schema+json"
			}
		}

		metadata: {
			category_id: {
				required: "optional"
				type:     "Category Parameters"
			}
		}

		payload: {
			description: """
				JSON Schema document which ensures the minimum required functional requirements
				of the Proposal Template are met.

				This ensures that payloads can be reliably interpreted by business logic processes, 
				while allowing for flexibility to capture extended information.
				"""
		}

		"signers": {
			roles: {
				// No User Role may publish this document.
				user: []

				// Root Admin and brand Admin may publish this document.
				admin: [
					"RootAdmin",
					"BrandAdmin",
				]
			}
		}

	}
}
