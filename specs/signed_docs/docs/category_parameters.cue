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
		payload: {
			schema: _ @embed(file="payload_schemas/category_parameters.schema.json")
			// Updated to reference the Category type from signed_doc.json
			type: "Category"
		}
		example: {
			// Example payload for documentation purposes
			payload: {
				parameter: "example_parameter"
				dataType: "String"
				defaultValue: "example_value"
				isRequired: true
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
		]
	}
}
