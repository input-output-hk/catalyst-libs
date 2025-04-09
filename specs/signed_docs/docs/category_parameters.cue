package signed_docs

// Proposal Document Definition

docs: #DocumentDefinitions & {
	"Category Parameters": {

		description: """
			Parameters which define a Category withing a Campaign under a Brand in the system.
			"""

		metadata: {
			campaign_id: {
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
		]}
}
