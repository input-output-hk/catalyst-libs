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

			parameters: {
				required: "yes"
				type: [
					"Brand Parameters",
					"Campaign Parameters",
					"Category Parameters",
				]
				linked_refs: [
					"ref",
					"template",
				]
			}
		}

		payload: {
			description: """
				JSON Schema document which defines the valid contents of a proposal document.
				"""
		}

		signers: {
			roles: {
				// No User Role may publish this document.
				user: []

				// Brand Admin and Lower may publish this document.
				admin: [
					"Brand Admin",
					"Campaign Admin",
				]
			}
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
