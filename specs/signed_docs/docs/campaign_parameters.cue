package signed_docs

// Import common definitions JSON schema for use in embedded schemas
_common_defs: {} @embed(file="./payload_schemas/common_definitions.schema.json")

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
			examples: [
				{
					title: "Final Proposal Submission"
					description: """
						This document indicates the linked proposal is final and requested to proceed for further consideration.
						"""
					example: {} @embed(file="./payload_schemas/campaign_parameter.example.json")
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
