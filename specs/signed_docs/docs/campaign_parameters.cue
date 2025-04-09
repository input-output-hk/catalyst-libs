package signed_docs

// Proposal Document Definition

docs: #DocumentDefinitions & {
	"Campaign Parameters": {
		description: """
			Parameters which define a Campaign within a Brand in the system.
			"""

		metadata: {
			brand_id: {
				required: "yes"
				type:     "Brand Parameters"
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
		]}
}
