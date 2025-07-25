// Template Json Schema Definitions Dictionary
// 
// All known and supported Json Schema definitions,
// and their parameters and documentation so that
// a dictionary document and the definitions themselves
// can be generated.
package form_template

#formElementDictionary: [string]: #formElement

// Definitions for all defined template schema field types.
dictionary: #formElementDictionary & {}

// Types of a Metadata Fields
#formTemplateElementNames: or([
	for k, _ in dictionary {k},
])

_defs: {
	for k, v in dictionary {"\(k)": v.definition}
}
