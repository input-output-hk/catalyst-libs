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

			brand_id: {
				required: "optional"
				type:     "Brand Parameters"
				linked_refs: [
					"ref",
					"template",
				]
			}
			campaign_id: {
				required: "optional"
				type:     "Campaign Parameters"
				linked_refs: [
					"ref",
					"template",
				]
			}
			category_id: {
				required: "optional"
				type:     "Category Parameters"
				linked_refs: [
					"ref",
					"template",
				]
			}
		}

		payload: {
			description: """
				JSON Document which must validate against the referenced template.
				"""
		}

		versions: [
			{
				version:  "0.01"
				modified: "2025-04-04"
				changes: """
					* First Published Version
					"""
			},
		]
	}
}
