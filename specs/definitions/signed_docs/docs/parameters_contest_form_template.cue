// Contest Form Template Document Definition
package signed_docs

import "text/template"

docs: "Contest Parameters Form Template": #generic_form_template & {
	_data: doc: "Contest Parameters"

	description: template.Execute(_form_template_description, _data)
	payload: description: template.Execute(_form_template_payload_description, _data)
	versions: _generic_form_template_versions
}
