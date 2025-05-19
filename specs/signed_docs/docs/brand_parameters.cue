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
				type:     "Brand Reference"
				validation: """
					Decisions reference brands, not the other way around. Ensure that any decision referencing this brand
					is consistent with the brand's parameters.
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
		signers: {
			role: "admin"
			description: "Only users with the admin role are authorized to publish brand parameters."
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
		example: {
			// Example payload for documentation purposes
			payload: {
				parameter: "example_brand"
				dataType: "String"
				defaultValue: "example_value"
				isRequired: true
			}
		}
	}
}
