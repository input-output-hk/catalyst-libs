// Template Json Schema Definitions Dictionary
// 
// All known and supported Json Schema definitions,
// and their parameters and documentation so that
// a dictionary document and the definitions themselves
// can be generated.
package signed_docs

import (
	"list"
	"regexp"
	"github.com/input-output-hk/catalyst-libs/specs/generic:optional"
)

#fieldType: *"string" |
	"object" |
	"array" |
	"integer" |
	"boolean"

_x_icon_choices: list.UniqueItems
_x_icon_choices: [
	"light-bulb",
	"view-grid",
	"top-bar",
	"tag",
	"chart-pie",
	"flag",
	"presentation-chart-line",
	"double_check",
]

#xIconChoices: or(_x_icon_choices)

_format_choices: list.Sort([
	"uri",
	"path",
], list.Ascending)

_content_choices: list.Sort([
	"text/plain",
	"text/markdown",
], list.Ascending)

_jsonParameter: {
	description: string
	required:    optional.#field
	type:        #fieldType
	choices?: [...string]
}

_regexTextPlain: "^.*$"
_regexHTTPSUrl:  "^https://[^\\s]+$"

// A JSON Schema field definition
#jsonSchemaFieldDefinition: {
	// Documentation
	description: string

	// MAPS to the `definitions` section within JSON Schema draft 7.
	definition: {
		"$comment":        description
		type:              #fieldType
		readOnly?:         bool & true // IF true, is not a data entry field, but can be used for presentation and formatting.
		format?:           string & or(_format_choices)
		contentMediaType?: string & or(_content_choices)
		pattern?:          regexp.Valid

		if type == "object" {
			additionalProperties: bool & false // IF false, can not define extra fields.
		}

		"x-note"?: string
	}

	parent?: #templateJsonSchemaDefNames

	// The parameters supported by a particular field definition
	parameters: {
		title?: _jsonParameter & {
			description: _ | *"The label attached to the field."
			required:    "yes"
		}
		description?: _jsonParameter & {
			description: _ | *"The description of the field presented during data entry."
			required:    "optional"
		}
		required?: _jsonParameter & {
			required: _ | *"optional"
		}
		default?: _jsonParameter & {
			required: _ | *"yes"
		}
		const?: _jsonParameter & {
			required: _ | *"yes"
		}
		properties?: _jsonParameter & {
			required:    _ | *"yes"
			description: _ | *"All sub fields contained in the object."
			required:    "yes"
		}
		minLength?: _jsonParameter & {
			type:        "integer"
			description: _ | *"Minimum number of characters allowed in the field."
			required:    "yes"
		}
		maxLength?: _jsonParameter & {
			type:        "integer"
			description: _ | *"Maximum number of characters allowed in the field."
			required:    "yes"
		}
		"x-guidance"?: _jsonParameter & {
			description: _ | *"Long form Markdown formatted description to give guidance about how the field is to be completed."
			required:    "optional"
		}
		"x-placeholder"?: _jsonParameter & {
			description: _ | *"Placeholder text to display inside the field if it is empty."
			required:    "optional"
		}
		"x-icon"?: _jsonParameter & {
			description: _ | *"The name of the Icon to display with the field."
			required:    "optional"
			choices:     _x_icon_choices
		}
		"x-order"?: _jsonParameter & {
			required: "yes"
			description: """
				The ordering of the properties to be enforced when displayed.
				Any field not listed here will get displayed in an arbitrary order.
				"""
		}
	}
}

#jsonSchemaFields: {
	[string]: #jsonSchemaFieldDefinition
}

// Types of a Metadata Fields
#templateJsonSchemaDefNames: or([
	for k, _ in templateJsonSchemaDefs {k},
])

// Definitions for all defined template schema field types.
templateJsonSchemaDefs: #jsonSchemaFields & {
	"schemaReferenceNonUI": {
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

	"segment": {
		description: """
			Logical Document Section - Grouping Only - Highest Level Group
			"""
		definition: {
			type: "object"
			"x-note": """
				Major sections of the proposal. Each segment contains sections of information grouped together.
				"""
		}
		parameters: {
			title: {
				description: "The title of the segment."
			}
			description: {
				description: "The displayable description attached to the segment.  Markdown formatted contents."
			}
			properties: {
				description: "The sub fields of the segment."
			}
			required: {
				description: "Which fields MUST appear in the segment."
				required:    "optional"
			}
			"x-icon": {}
			"x-order": {}
		}
	}

	"section": {
		description: "UI - Logical Document Sub-Section Break."
		parent:      "segment"
		definition: {
			type: "object"
			"x-note": """
				Subsections containing specific details about the proposal.
				"""
		}
		parameters: {
			title: {
				description: "The title of the section."
				required:    "yes"
			}
			description: {
				description: "The displayable description attached to the section.  Markdown formatted contents."
			}
			properties: {
				description: "The sub fields of the section."
			}
			required: {
				description: "Which fields MUST appear in the segment."
				required:    "optional"
			}
			"x-icon": {}
			"x-order": {}
		}
	}

	"singleLineTextEntry": {
		description: "UI - Single Line text entry without any markup or rich text capability."
		definition: {
			type:             "string"
			contentMediaType: "text/plain"
			pattern:          _regexTextPlain
			"x-note": """
				Enter a single line of text.
				No formatting, line breaks, or special characters are allowed.
				"""
		}
		parameters: {
			title: {}
			description: {}
			minLength: {}
			maxLength: {}
			"x-guidance": {}
			"x-placeholder": {}
		}
	}

	"singleLineHttpsURLEntry": {
		description: "UI - Single Line text entry for HTTPS Urls."
		definition: {
			type:    "string"
			format:  "uri"
			pattern: _regexHTTPSUrl
			"x-note": """
				Must start with 'https://' and is followed by one or more 
				non-whitespace characters, ending at the end of the string.
				"""
		}
		parameters: {
			title: {}
			description: {}
			minLength: {}
			maxLength: {}
			"x-guidance": {}
			"x-placeholder": {}
		}
	}

}
