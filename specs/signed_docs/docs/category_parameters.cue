package signed_docs

// Import common definitions JSON schema for use in embedded schemas
_common_defs: {} @embed(file="./payload_schemas/common_definitions.schema.json")

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
			examples: [
				{
					title: "Final Proposal Submission"
					description: """
						This document indicates the linked proposal is final and requested to proceed for further consideration.
						"""
					example: {} @embed(file="./payload_schemas/category_parameter.example.json")
				},
			]
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
