package signed_docs

// Proposal Document Definition

docs: #DocumentDefinitions & {
	"Campaign Parameters": {
		description: """
			Parameters which define a Campaign within a Brand in the system.
			"""
		headers: "content type": value: "application/json"

		metadata: parameters: {
			required: "yes"
			type:     "Brand Parameters"
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
