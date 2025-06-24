// Signed Document Definitions
// 
// Metadata Types and Constraints
package signed_docs

import (
	"strings"
)

_metadata: _parameters: {
	_description: """
		A reference to the Parameters Document this document lies under.
		"""

	_validation: """
		In addition to the validation performed for `Document Reference` type fields: 

		* Any linked referenced document that includes a `parameters` metadata must match the 
		`parameters` of the referencing document,
		or a parent of those `parameters`.

		For example, a linked reference to `Contest Parameters` is transitively a reference to
		the Parameters document it references, and each parameters document they reference 
		until the `Brand` parameters document is reached.

		The use case here is for Templates.
		The profile template, or proposal templates could be defined at any of these
		levels, and as long as they all refer to the same chain of parameters in the
		hierarchy they are all valid.
		"""
}

#metadata: parameters: {

	description: string | *"""
			\(_metadata._parameters._description)
			"""
	validation:  string | *"""
			\(_metadata._parameters._validation)
			"""
}

// List of all Parameters Docs (not templates or actions)
#parameterDocNamesList: [...string] & [
	for k, _ in _allDocs
	if strings.Contains(k, "Parameter") &&
		!strings.Contains(k, "Template") &&
		!strings.Contains(k, "Action") {k},
]

// Top Level general documentation for Parameters Metadata.
metadata: headers: parameters: {
	required: "optional"
	type:     #parameterDocNamesList
}
