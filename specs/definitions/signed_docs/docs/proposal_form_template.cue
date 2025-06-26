// Proposal Form Template Document Definition
package signed_docs

import "text/template"

docs: "Proposal Form Template": #generic_form_template & {
	_data: doc: "Proposal"

	description: template.Execute(_form_template_description, _data)
	metadata: parameters: _metadataFieldSystemParameters
	payload: description: template.Execute(_form_template_payload_description, _data)
	versions: _generic_form_template_versions
}
