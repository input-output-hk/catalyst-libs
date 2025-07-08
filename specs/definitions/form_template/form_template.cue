// Template Json Schema Definitions Dictionary
// 
// All known and supported Json Schema definitions,
// and their parameters and documentation so that
// a dictionary document and the definitions themselves
// can be generated.
package form_template

// Generic Form Template
formTemplate: {
	$schema:     "https://json-schema.org/draft/2020-12/schema"
	title:       "Form Template"
	description: "Generic Template Schema."
	maintainers: [{
		name: "Catalyst Team"
		url:  "https://projectcatalyst.io/"
	}]
	type: "object"
	properties: {}
	additionalProperties: false

	$defs: _defs
}
