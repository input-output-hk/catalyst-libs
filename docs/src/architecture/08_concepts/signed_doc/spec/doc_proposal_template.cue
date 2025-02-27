package signed_docs

// Proposal Document Definition

docs: #DocumentDefinitions & {
	"Proposal Template": {
		description: """
			## Proposal Template Document
			
			A Proposal Template defines the allowed payload contents of a
			linked proposal.

			Proposals themselves are intentionally general, however they may be
			linked to a brand/campaign or category via the template used by the proposal.

			The payload of a proposal is controlled by its template.
			"""

		headers: {
			"content type": {
				value: "application/schema+json"
			}
		}

		metadata: {
			template: {
				required: "optional"
				type:     "Proposal Meta Template"
			}

			category_id: {
				required: "optional"
				type:     "Category Parameters"
			}
		}
	}
}
