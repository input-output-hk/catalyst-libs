// Brand Parameters Document Definition
package signed_docs

import "text/template"

docs: "Brand Parameters": #generic_parameters & {

	_data: {
		doc:        "Brand"
		doc_parent: "Brand"
	}

	description: template.Execute(_parameters_description, _data)
	validation:  "No extra validation defined."
	metadata: {
		template: type:       "\(_data.doc) Parameters Form Template"
		parameters: required: "excluded"
	}
	payload: description: template.Execute(_parameters_payload_description, _data)
	versions: _generic_parameters_versions
}
