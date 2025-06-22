// Proposal Form Template Document Definition
package signed_docs

import "text/template"

_proposal_comment_presentation_data: doc: "Proposal Comment"

docs: #DocumentDefinitions & {
	"Proposal Comment Presentation Template": _generic_presentation_template
	"Proposal Comment Presentation Template": description: template.Execute(_presentation_template_description, _proposal_comment_presentation_data)
	"Proposal Comment Presentation Template": payload: description: template.Execute(_presentation_template_payload_description, _proposal_comment_presentation_data)
	"Proposal Comment Presentation Template": versions: _generic_presentation_template_versions
}
