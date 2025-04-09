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

			brand_id: {
				required: "optional"
				type:     "Brand Parameters"
				linked_refs: [
					"template",
				]
			}
			campaign_id: {
				required: "optional"
				type:     "Campaign Parameters"
				linked_refs: [
					"template",
				]
			}
			category_id: {
				required: "optional"
				type:     "Category Parameters"
				linked_refs: [
					"template",
				]
			}
		}

		payload: {
			description: """
				Proposal Document drafted for submission to a category of a campaign.

				Must be valid according to the schema contained within the 
				`Document Reference` from the `template` metadata.
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
