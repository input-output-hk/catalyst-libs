// Contest Parameters Document Definition
package signed_docs

import "text/template"

docs: "Contest Parameters": #generic_parameters & {

	_data: {
		doc:        "Contest"
		doc_parent: "Brand/Campaign/Category"
	}

	description: template.Execute(_parameters_description, _data)
	validation:  template.Execute(_parameters_parent_validation, _data)
	metadata: {
		template: type: "\(_data.doc) Parameters Form Template"
		parameters: {
			required: "yes"
			type:     doc_clusters."System Parameters".docs
		}
	}
	payload: description: template.Execute(_parameters_payload_description, _data)
	versions: _generic_parameters_versions
}
