// Proposal Form Template Document Definition
package signed_docs

import "text/template"

_proposal_form_data: doc: "Proposal"

docs: #DocumentDefinitions & {
	"Proposal Form Template": #generic_form_template & {
		description: template.Execute(_form_template_description, _proposal_form_data)
		payload: description: template.Execute(_form_template_payload_description, _proposal_form_data)
		versions: _generic_form_template_versions
		//metadata: template: type: "Proposal Presentation Template"
	}
}
