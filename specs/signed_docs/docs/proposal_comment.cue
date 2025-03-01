package signed_docs

// Proposal Document Definition

docs: #DocumentDefinitions & {
	"Proposal Comment": {
		description: """
			## Proposal Comment Document
			
			A Proposal Comment is a document which comments on a referenced Proposal document.
			
			Proposal Comments themselves are intentionally general, however they may be
			linked to a brand/campaign or category via the template used by the proposal.

			The payload of a proposal comment is controlled by its template.
			"""

		metadata: {
			ref: {
				required: "yes"
				type:     "Proposal"
			}

			reply: {
				required: "optional"
				type:     "Proposal Comment"
			}

			section: {
				required: "optional"
			}

			template: {
				required: "yes"
				type:     "Proposal Comment Template"
			}

			category_id: {
				required: "optional"
				type:     "Category Parameters"
			}
		}

		payload: {
			description: """
				JSON Document which must validate against the referenced template.
				"""
		}

	}
}
