// Signed Document Definitions
// 
// Metadata Types and Constraints
package signed_docs

import (
	"github.com/input-output-hk/catalyst-libs/specs/signed_doc_types"
)

_template_description: """
	Reference to the template used to create and/or validate this document.
	"""
_template_validation: """
	In addition to the validation performed for `Document Reference` type fields, 
	The document payload is not valid if it does not validate completely against the referenced template.
	"""

#metadata: template: {
	description: _template_description
	validation:  _template_validation
}

// Note: we make all normally excluded fields optional at the global level, because they are globally optional
metadata: headers: {
	template: required: "optional"
	template: type:     signed_doc_types.templateDocNamesList
}
