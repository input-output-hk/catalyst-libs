package signed_docs

// Import common definitions JSON schema for use in embedded schemas
_common_defs: {} @embed(file="./payload_schemas/common_definitions.schema.json")

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
			schema: {} @embed(file="./payload_schemas/decision_parameters.schema.json")
			examples: [
				{
					title: "Final Proposal Submission"
					description: """
						This document indicates the linked proposal is final and requested to proceed for further consideration.
						"""
					example: {} @embed(file="./payload_schemas/decision_parameter.example.json")
				},
			]}
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
