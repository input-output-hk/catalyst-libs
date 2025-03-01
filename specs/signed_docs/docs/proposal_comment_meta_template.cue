package signed_docs

// Proposal Document Definition

docs: #DocumentDefinitions & {
	"Proposal Comment Meta Template": {
		description: """
			## Proposal Comment Meta Template Document
			
			A Proposal Comment Meta Template is used to enforce functional requirements
			are met in any Proposal Comment Template.

			The payload of a proposal comment template is controlled by its meta template.
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
				of the Proposal Comment Template are met.

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
