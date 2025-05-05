package signed_docs

// Proposal Document Definition

docs: #DocumentDefinitions & {
	"Election Parameters": {
		description: """
			Parameters which define an individual voting event.
			"""

		metadata: {
			parameters: {
				required: "yes"
				type: [
					"Brand Parameters",
					"Campaign Parameters",
					"Category Parameters",
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
		]}
}
