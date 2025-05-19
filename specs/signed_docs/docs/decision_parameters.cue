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
				type:     doc_clusters."System Parameters".docs
				validation: """
					Decisions must reference brands. Ensure that the brand referenced by this decision
					is valid and consistent with the decision's parameters.
				"""
			}
		}
		payload: {
			schema: _ @embed(file="payload_schemas/decision_parameters.schema.json")
		}
		example: {
			// Example payload for documentation purposes
			payload: {
				action: "final"
				brandReference: "example_brand_id"
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
