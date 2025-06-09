// Definitions of the field parameters within a template.
package template_field_definitions

import (
	"list"
	"github.com/input-output-hk/catalyst-libs/specs/generic:optional"
)

// Supported field `type`
// Same as Json Schema minus `null`
#fieldType: "array" |
	"array" |
	"boolean" |
	"integer" |
	"number" |
	"object" |
	*"string"

// List of properties allowed in the `property` field of `jsonParameter`
#properties: "oneOf"

#jsonParameter: {
	property?:   #properties // Name of the property, IF its not the same as the parameter.
	description: string
	required:    optional.#field
	type:        #fieldType
	items?:      #jsonParameter
	choices?:    list.UniqueItems
}

// The parameters supported by a particular field definition
_allParameters: {
	title?: #jsonParameter & {
		description: _ | *"The label attached to the field."
		required:    "yes"
	}
	description?: #jsonParameter & {
		description: _ | *"The description of the field presented during data entry."
		required:    "optional"
	}
	required?: #jsonParameter & {
		required: _ | *"optional"
	}
	default?: #jsonParameter & {
		required: _ | *"yes"
	}
	const?: #jsonParameter & {
		required: _ | *"yes"
	}
	properties?: #jsonParameter & {
		required:    _ | *"yes"
		description: _ | *"All sub fields contained in the object."
		required:    "yes"
	}
	minLength?: #jsonParameter & {
		type:        "integer"
		description: _ | *"Minimum number of characters allowed in the field."
		required:    "optional"
	}
	maxLength?: #jsonParameter & {
		type:        "integer"
		description: _ | *"Maximum number of characters allowed in the field."
		required:    "yes"
	}
	enum?: #jsonParameter & {
		type: "array"
		items: {
			description: "An element of the Enum."
			type:        "string"
		}
		description: _ | *"An array of string to select from."
		required:    "yes"
	}
	minItems?: #jsonParameter & {
		type:        "integer"
		description: _ | *#"An array instance is valid against "minItems" if its size is greater than, or equal to, the value of this keyword."#
		required:    "optional"
	}
	maxItems?: #jsonParameter & {
		type:        "integer"
		description: _ | *#"An array instance is valid against "maxItems" if its size is less than, or equal to, the value of this keyword."#
		required:    "yes"
	}
	contains?: #jsonParameter & {
		type: "array"
		items: {
			description: "An individual Choice."
			type:        "string"
		}
		description: _ | *"The choices the multi select can contain."
		required:    "yes"
	}
	oneOf_groupedTags?: #jsonParameter & {
		property: "oneOf"
		type:     "array"
		items: {
			description: """
					An array of grouped tag objects, of which one can be selected.
					Each object *MUST* have the form:
					
					```json
					"properties": {
						"group": {
							"$ref": "#/definitions/tagGroup",
							"const": <group name string>
						},
						"tag": {
							"$ref": "#/definitions/tagSelection",
							"enum": [
								<tag 1 string>,
								<tag 2 string>,
								...
							]
						}
					}
					```
				"""
			type: "object"
		}
		description: "A set of tags with a group selector."
	}
	"x-guidance"?: #jsonParameter & {
		description: _ | *"Long form Markdown formatted description to give guidance about how the field is to be completed."
		required:    "optional"
	}
	"x-placeholder"?: #jsonParameter & {
		description: _ | *"Placeholder text to display inside the field if it is empty."
		required:    "optional"
	}
	"x-icon"?: #jsonParameter & {
		description: _ | *"The name of the Icon to display with the field."
		required:    "optional"
		choices:     _allIcons
	}
	"x-order"?: #jsonParameter & {
		required: "yes"
		description: """
			The ordering of the properties to be enforced when displayed.
			Any field not listed here will get displayed in an arbitrary order.
			"""
	}
	"x-subsection"?: #jsonParameter & {
		required: "optional"
		type:     bool
		description: """
			TODO: Explain what this parameter does...
			"""
	}
}
