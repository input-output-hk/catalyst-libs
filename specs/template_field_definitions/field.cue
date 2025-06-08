// Individual Field Definition.
package template_field_definitions

import (
	"regexp"
)

#fieldDefinition: {
	$comment:  "See the Template Schema Definition Dictionary for detailed documentation."
	type:      #fieldType
	readOnly?: true // IF true, is not a data entry field, but can be used for presentation and formatting.

	if type == "object" {
		additionalProperties: false // IF false, can not define extra fields.
	}

	if type == "array" {
		items:        #fieldDefinition
		uniqueItems?: true
	}

	if type == "string" {
		format?:           #formatChoices
		contentMediaType?: #contentMediaTypeChoices
		pattern?:          regexp.Valid
	}

}

// A JSON Schema field definition
#jsonSchemaFieldDefinition: {
	// Documentation
	description: string

	// MAPS 1:1 to the `definitions` section within JSON Schema draft 7.
	definition: #fieldDefinition

	// If this field type must appear only as a child of another field type.
	parent?: #templateJsonSchemaDefNames

	// The parameters supported by a particular field definition
	parameters: _allParameters
}

#jsonSchemaFields: [string]: #jsonSchemaFieldDefinition
