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

		headers: "content type": value: "application/schema+json"

		metadata: parameters: {
			required: "yes"
			type:     doc_clusters."System Parameters".docs
		}

		payload: {
			description: """
				JSON Schema document which ensures the minimum required functional requirements
				of the Proposal Template are met.

				This ensures that payloads can be reliably interpreted by business logic processes, 
				while allowing for flexibility to capture extended information.
				"""

			schema: "https://json-schema.org/draft-07/schema"
		}
		signers: roles: {
			// No User Role may publish this document.
			user: []

			// Root Admin and brand Admin may publish this document.
			admin: [
				"Root Admin",
				"Brand Admin",
			]
		}

		versions: [
			{
				version:  "0.01"
				modified: "2025-04-04"
				changes: """
					* First Published Version
					"""
			},
			{
				version:  "0.03"
				modified: "2025-05-05"
				changes: """
					* Use generalized parameters.
					"""
			},
		]
	}
}
