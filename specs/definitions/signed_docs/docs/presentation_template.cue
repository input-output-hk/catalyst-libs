package signed_docs

import (
	"github.com/input-output-hk/catalyst-libs/specs/signed_doc_types"
)

// Template Standardized Descriptions

_presentation_template_description: """
	A Presentation Template defines how the data
	captured by the *ANY* Set of Form Template or system dynamic data is to be displayed.

	Multiple Presentation Templates will exist, and will be associated with multiple document types. 
	There is no strict 1:1 relationship between any document, its template and a presentation template.

	The presentation templates are context sensitive, and each template defines the sources
	of published document information they require.

	Presentation Templates can reference any data contained
	in the referenced Documents, as well as any documents linked by:

	* `ref`
	* `reply`
	* `parameters`

	The presentation of the payload of all data when not capturing or displaying a 
	Form via its Form Template is controlled by one or more Presentation Template documents.
	"""

_presentation_template_payload_description: """
	The Presentation Template is defined by its schema.
	See `[presentation_templates.md](TODO)`
	"""

docs: #DocumentDefinitions & {
	"Presentation Template": {
		description: _presentation_template_description

		headers: "content type": value: "application/schema+json"

		metadata: parameters: {
			required: "yes"
			type:     signed_doc_types.doc_clusters."System Parameters".docs
		}

		payload: description: _presentation_template_payload_description
		payload: schema:      presentationTemplate.schema

		signers: roles: {
			// No User Role may publish this document.
			user: []

			// Brand Admin and Lower may publish this document.
			admin: [
				"Brand Admin",
				"Campaign Admin",
			]
		}

		versions: [
			{
				version:  "0.0.4"
				modified: "2025-05-05"
				changes: """
					* First Version.
					"""
			},
			{
				version:  "0.1.0"
				modified: "2025-07-30"
				changes: """
					* Updated to match Presentation Schema Definitions.
					"""
			},
		]
	}}
