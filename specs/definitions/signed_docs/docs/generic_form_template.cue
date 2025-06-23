// Template Standardized Descriptions
package signed_docs

_form_template_description: """
	A {{ .doc }} Form Template defines both:
	
	* The data that is entered in the Form.
	* Formatting hints for the collection of the data in a form.

	A {{ .doc }} Form Template is a JSON Schema Document.

	{{ .doc }} entry *SHOULD* use the hints when collecting 
	data defined by the {{ .doc }} Form Template to provide a 
	consistent user interface.
	It *CAN* also use those hints when re-displaying the full forms data.

	Alternatively a {{ .doc }} Presentation Template can be used to
	format the {{ .doc }} data for presentation.

	The {{ .doc }} Document is intentionally general, 
	however it may be linked to a brand/campaign or category 
	via the template used by the {{ .doc }}.

	The payload of a {{ .doc }} is controlled by its template.
	"""

_form_template_payload_description: """
	JSON Schema document which defines the valid contents and 
	formatting hints for the collection of data for a 
	{{ .doc }} Document.
	"""

_metadataFieldSystemParameters: #metadataField & {
	// Is the field required to be present.
	required: "yes"
	type:     doc_clusters."System Parameters".docs
}

#generic_form_template: #signedDocument & {
	description: _

	headers: "content type": value: "application/schema+json"

	metadata: template: required: "excluded"

	payload: description: _

	signers: roles: {
		// No User Role may publish this document.
		user: []

		// Brand Admin and Lower may publish this document.
		admin: _ | *[
			"Brand Admin",
		]
	}

}

_generic_form_template_versions: [#changelogEntry, ...#changelogEntry] & [
	{
		version:  "0.01"
		modified: "2025-04-04"
		changes: """
			* First Published Version
			"""
	},
	{
		version:  "0.03"
		modified: "2025-05-05"
		changes: """
			* Use generalized parameters.
			"""
	},
	{
		version:  "0.04"
		modified: "2025-05-05"
		changes: """
			* Generalize the Form Template definitions.
			"""
	},
]
