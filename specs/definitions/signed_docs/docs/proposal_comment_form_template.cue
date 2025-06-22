// Proposal Comment Form Template Document Definition
package signed_docs

import "text/template"

_proposal_comment_form_data: doc: "Proposal Comment"

docs: #DocumentDefinitions & {
	"Proposal Comment Form Template": #generic_form_template & {
		description: template.Execute(_form_template_description, _proposal_comment_form_data)
		payload: description: template.Execute(_form_template_payload_description, _proposal_comment_form_data)
		versions: _generic_form_template_versions
	}
}
