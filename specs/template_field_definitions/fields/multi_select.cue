// Radio Button selection from a list of text options.

package template_field_definitions

dictionary: #jsonSchemaFields & {
	multiSelect: {
		description: """
			UI - Multiselect from the given items.

			Select multiple options from the dropdown menu.
			Multiple choices are allowed.
			All choices MUST be unique.
			"""
		definition: {
			type:        "array"
			uniqueItems: true
			items:       dictionary.singleLineTextEntry.definition
		}
		parameters: {
			title: {}
			description: {}
			default: {
				description: "Default selections can be supplied."
				required:    "optional"
			}
			minItems: {}
			maxItems: {}
			contains: {}
			"x-guidance": {}
		}
	}
}
