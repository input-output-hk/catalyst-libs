// Individual Field Definition.
package form_template

import (
	"regexp"
)

// Schema Definition for the Form Element.
// This appears in the `definitions` section of the Form Template
// exactly as defined here.
// UI *MUST* not process definitions, its only use is to enforce validation.
// UI/UX *MUST* interpret the name of the Form Element via `$ref` only.
// The values in the definition can inform creation of a valid UI widget
// which matches the Form Element, but that *MUST* not be dynamic.
#formElementDefinition: {
	// The underlying json type of the Form Element.
	type: #fieldType

	// IF true, the Form Element is not a data entry field.
	// Depending on the Form Element, it may not be displayed by the form.
	// but MUST be present in the encoded json form data.but can be used for presentation and formatting.
	readOnly?: true

	// Form Elements which hold other form elements
	if type == "object" {
		// MUST not allow undefined properties
		additionalProperties: false // IF false, can not define extra fields.
		// MAY list properties as non optional by including them in a `required` list.
		// properties not included in this list are optional.
		// may not be entered by the user into the form, and if not entered
		// should be excluded (or defaulted as required) from the 
		// forms json data.
		required?: [...string]
	}

	// Form Elements which are a list of other Form Elements
	if type == "array" {
		// The type of Form Elements in the list.
		items: #formElementDefinition
		// Are the Elements Unique?
		uniqueItems: false | *true
	}

	if type == "string" {
		format?:           #formatChoices
		contentMediaType?: #contentMediaTypeChoices
		pattern?:          regexp.Valid
		minLength?:        int // Only to enforce that the field can not be empty (such as when used in lists)
	}

}

// An Element which can be present in a Templated Form.
// Elements define both the data validation and the expected
// type of data entry UI expected of the Element.
// This allows JsonSchema to express both Validation and Presentation without
// requiring an extensive new DSL to be created.
#formElement: {
	// Documentation
	description: string

	// MAPS 1:1 to the `definitions` section within JSON Schema draft 7.
	definition: #formElementDefinition

	// If this field type must appear only as a child of another field type.
	// If `parent` is NOT defined, then the `parent` is the root of the template.
	// The root object of the template is defined with the special string `{}`
	// 
	parent: #formTemplateElementNames | *"{}"

	// The parameters supported by a particular field definition
	parameters: _allParameters
}

#formTemplate: [string]: #formElement
