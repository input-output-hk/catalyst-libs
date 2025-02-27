package signed_docs

// Proposal Document Definition

docs: #DocumentDefinitions & {
	"Proposal": {
		description: """
			## Proposal Document
			
			A Proposal is a document which describes a proposed solution or project to
			address the criteria of a category within a campaign.
			
			The proposal itself is a draft document, it is not submitted for consideration
			unless a `Proposal Submission Action` is submitted which references it.

			Proposals themselves are intentionally general, however they may be
			linked to a brand/campaign or category via the template used by the proposal.

			The payload of a proposal is controlled by its template.
			"""

		metadata: {
			template: {
				required: "yes"
				type:     "Proposal Template"
			}

			category_id: {
				required: "optional"
				type:     "Category Parameters"
			}
		}
	}
}
