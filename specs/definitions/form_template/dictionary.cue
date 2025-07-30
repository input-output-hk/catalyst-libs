// Template Json Schema Definitions Dictionary
// 
// All known and supported Json Schema definitions,
// and their parameters and documentation so that
// a dictionary document and the definitions themselves
// can be generated.
package form_template

import (
	"github.com/input-output-hk/catalyst-libs/specs/regex"
)

#elementName: string
#elementName: =~regex.def.camelCaseName.pattern

#formElementDictionary: [#elementName]: #formElement

// Definitions for all defined template schema field types.
dictionary: #formElementDictionary & {}

// Types of a Metadata Fields
_formTemplateElementNames: [
	for k, _ in dictionary {k},
]

#formTemplateElementName: or(_formTemplateElementNames)

// A single form elements parent element, or the template root `{}`
#formTemplateElementParent: or([for x in _formTemplateElementNames {x}, "{}"]) | *"{}"

// One or more parents that a form element may have.
#formTemplateElementParents: [...#formTemplateElementParent] | *["{}"]

_defs: {
	for k, v in dictionary {"\(k)": v.definition}
}
