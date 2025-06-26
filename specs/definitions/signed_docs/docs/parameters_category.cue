// Category Parameters Document Definition
package signed_docs

import "text/template"

docs: "Category Parameters": #generic_parameters & {

	_data: {
		doc:        "Category"
		doc_parent: "Campaign"
	}

	description: template.Execute(_parameters_description, _data)
	validation:  template.Execute(_parameters_parent_validation, _data)
	metadata: {
		template: type: "\(_data.doc) Parameters Form Template"
		parameters: {
			required: "yes"
			type:     "\(_data.doc_parent) Parameters"
		}
	}
	payload: description: template.Execute(_parameters_payload_description, _data)
	versions: _generic_parameters_versions
}
