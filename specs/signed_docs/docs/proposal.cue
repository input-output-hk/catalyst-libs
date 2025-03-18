package signed_docs

// Proposal Document Definition

docs: #DocumentDefinitions & {
	"Proposal": {
		description: """
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

			collaborators: {
				required: "optional"
			}

			category_id: {
				required: "optional"
				type:     "Category Parameters"
			}
		}

		payload: {
			description: """
				Proposal Document drafted for submission to a category of a campaign.

				Must be valid according to the schema of the referenced Template.
				"""
		}

		signers: {
			roles: {
				user: [
					"Proposer",
				]
			}

			update: {
				"collaborators": true
			}
		}

		authors: {
			"Steven Johnson": "steven.johnson@iohk.io"
		}
	}
}
