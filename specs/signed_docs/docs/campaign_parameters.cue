package signed_docs

// Campaign Parameters Document Definition
docs: #DocumentDefinitions & {
	"Campaign Parameters": {
		description: """
			Parameters which define a Campaign within a Brand in the system.
			"""
		metadata: {
			parameters: {
				required: "yes"
				type:     "Campaign Parameters"
			}
			collaborators: {
				required: "optional"
			}
		}

		payload: {
			description: "Payload for Campaign Parameters document."
			schema: {} @embed(file="./payload_schemas/campaign_parameters.schema.json")
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
