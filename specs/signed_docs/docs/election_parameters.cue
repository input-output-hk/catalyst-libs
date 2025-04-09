package signed_docs

// Proposal Document Definition

docs: #DocumentDefinitions & {
	"Election Parameters": {
		description: """
			Parameters which define an individual voting event.
			"""

		metadata: {
			brand_id: {
				required: "yes"
				type:     "Brand Parameters"
			}
			campaign_id: {
				required: "yes"
				type:     "Campaign Parameters"
			}
			category_id: {
				required: "yes"
				type:     "Category Parameters"
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
