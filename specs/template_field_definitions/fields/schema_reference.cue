// Schema Reference - Not used in any UI
package template_field_definitions

dictionary: #jsonSchemaFields & {
	schemaReferenceNonUI: {
		description: """
			Enforces the json document having a proper reference to the schema.
			"""
		definition: {
			type:     "string"
			readOnly: true
		}

		parameters: {
			default: {
				description: "The path that is defined to find the matching json schema."
				required:    "yes"
			}
			const: {
				description: "The path must always be this value. **MUST** match `default`."
				required:    "yes"
			}
		}
	}
}
