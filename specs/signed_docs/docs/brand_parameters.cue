package signed_docs

// Brand Parameters Document Definition

docs: #DocumentDefinitions & {
	"Brand Parameters": {
		description: """
			Parameters which define this brand within the system.
			"""
		metadata: {
			parameters: {
				required: "optional"
				type:     "Brand Parameters"
			}
			collaborators: {
				required: "optional"
			}
		}
		signers: _allowedSigners
		versions: [
			{
				version:  "0.01"
				modified: "2025-04-04"
				changes: """
						* First Published Version
					"""
			},
		]
		payload: {
			description: "Payload for Brand Parameters document."
			schema: {} @embed(file="./payload_schemas/brand_parameters.schema.json")
			examples: [
				{
					title: "Final Proposal Submission"
					description: """
						This document indicates the linked proposal is final and requested to proceed for further consideration.
						"""
					example: {} @embed(file="./payload_schemas/brand_parameter.example.json")
				},
			]
		}
	}
}
