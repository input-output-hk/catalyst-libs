package signed_docs

// Decision Parameters Document Definition
docs: #DocumentDefinitions & {
	"Decision Parameters": {
		description: """
				Parameters which define an individual voting event.
			"""
		metadata: {
			parameters: {
				required: "yes"
				type:     "Decision Parameters"
			}
		}
		payload: {
			description: "Payload for Decision Parameters document."
			schema:      _ @embed(file="./payload_schemas/decision_parameters.schema.json")
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
			{
				version:  "0.04"
				modified: "2025-06-06"
				changes: """
						* Added brand reference to decision parameters.
					"""
			},
		]
	}
}
