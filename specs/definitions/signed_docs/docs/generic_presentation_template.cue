package signed_docs

// Template Standardized Descriptions

_presentation_template_description: """
	A {{ .doc }} Presentation Template defines how the data
	captured by the {{ .doc }} Form Template is to be displayed.

	Multiple {{ .doc }} Presentation Templates can exist for the 
	same {{.doc }} Form Template.
	Each can be used to display the form data under different 
	circumstances.

	{{ .doc }} Presentation Templates can reference any data contained
	in the {{ .doc }} Document, as well as any documents linked by:

	* `ref`
	* `reply`
	* `parameters`

	The presentation of the payload of a {{ .doc }} is controlled by 
	its {{ .doc }} Presentation Template/s.
	"""

_presentation_template_payload_description: """
	TBD.
	But roughly, will be:
	
	1. A way to identify where the presentation template is intended to be used.
	2. Optional CSS to control the presentation.
	3. A Handlebars templated HTML or Markdown file data which defines the presentation.
	"""

#generic_presentation_template: #signedDocument & {
	description: _

	headers: "content type": value: "application/schema+json"

	metadata: parameters: {
		required: "yes"
		type:     doc_clusters."System Parameters".docs
	}

	payload: description: _

	signers: roles: {
		// No User Role may publish this document.
		user: []

		// Brand Admin and Lower may publish this document.
		admin: [
			"Brand Admin",
			"Campaign Admin",
		]
	}
}

_generic_presentation_template_versions: [#changelogEntry, ...#changelogEntry] & [
	{
		version:  "0.04"
		modified: "2025-05-05"
		changes: """
			* First Version.
			"""
	},
]
