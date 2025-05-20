package signed_docs

// Category Parameters Document Definition
docs: #DocumentDefinitions & {
	"Category Parameters": {
		description: """
			Parameters which define a Category within a Campaign under a Brand in the system.
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
			description: "Payload for Category Parameters document."
			schema: {} @embed(file="./payload_schemas/category_parameters.schema.json")
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
