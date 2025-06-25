// Proposal Form Template Document Definition
package signed_docs

import "text/template"

docs: "Proposal Presentation Template": #generic_presentation_template & {
	_data: doc: "Proposal"

	description: template.Execute(_presentation_template_description, _data)
	payload: description: template.Execute(_presentation_template_payload_description, _data)
	versions: _generic_presentation_template_versions
}
