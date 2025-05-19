package signed_docs

import "encoding/json"

// Brand Parameters Document Definition

docs: #DocumentDefinitions & {
	"Brand Parameters": {
		description: """
			Parameters which define this brand within the system.
			"""
		metadata: {
			parameters: {
				required: "optional"
				type:     "Decision Parameters"
				validation: """
					In addition to the validation performed for `Document Reference` type fields: 
					* Any linked referenced document that includes a `parameters` metadata must match the 
					  `parameters` of the referencing document.
				"""
			}
			collaborators: {
				required: "optional"
				validation: """
					This list does not imply these collaborators have consented to collaborate, only that the author/s
					are permitting these potential collaborators to participate in the drafting and submission process.
					However, any document submission referencing a proposal MUST be signed by all collaborators in
					addition to the author.
				"""
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
		]
		payload: {
			// Load and integrate the schema for Brand Parameters
			schema: *json.Unmarshal(_ @file("payload_schemas/brand_parameters.schema.json")) | _
			// Updated to reference the Brand type from signed_doc.json
			type: "Brand"
		}
	}
}
