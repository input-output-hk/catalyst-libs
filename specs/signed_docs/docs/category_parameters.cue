package signed_docs

// Proposal Document Definition

docs: #DocumentDefinitions & {
	"Category Parameters": {

		description: """
			Parameters which define a Category withing a Campaign under a Brand in the system.
			"""

		metadata: {
			parameters: {
				required: "yes"
				type:     "Campaign Parameters"
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
