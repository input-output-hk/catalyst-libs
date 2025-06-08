package signed_docs

// Proposal Document Definition

docs: #DocumentDefinitions & {
	"Decision Parameters": {
		description: """
			Parameters which define an individual voting event.
			"""

		metadata: parameters: {
			required: "yes"
			type:     doc_clusters."System Parameters".docs
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
		]}
}
