// Proposal Form Template Document Definition
package signed_docs

import "text/template"

_proposal_presentation_data: doc: "Proposal"

docs: #DocumentDefinitions & {
	"Proposal Presentation Template": _generic_presentation_template
	"Proposal Presentation Template": description: template.Execute(_presentation_template_description, _proposal_form_data)
	"Proposal Presentation Template": payload: description: template.Execute(_presentation_template_payload_description, _proposal_form_data)
	"Proposal Presentation Template": versions: _generic_presentation_template_versions
}
