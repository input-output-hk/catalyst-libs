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
	}
}
