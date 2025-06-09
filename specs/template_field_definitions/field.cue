// Individual Field Definition.
package template_field_definitions

import (
	"regexp"
)

// Note: only attributes which make sense for and are used by definitions
// are defined.  This should not list any attribute which would needlessly
// restrict the definition, and should otherwise be a parameter set by the
// template itself.
#fieldDefinition: {
	$comment:  "See the Template Schema Definition Dictionary for detailed documentation."
	type:      #fieldType
	readOnly?: true // IF true, is not a data entry field, but can be used for presentation and formatting.

	if type == "object" {
		additionalProperties: false // IF false, can not define extra fields.
		required?: [...string]
	}

	if type == "array" {
		items:        #fieldDefinition
		uniqueItems?: true
	}

	if type == "string" {
		format?:           #formatChoices
		contentMediaType?: #contentMediaTypeChoices
		pattern?:          regexp.Valid
		minLength?:        int // Only to enforce that the field can not be empty (such as when used in lists)
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
