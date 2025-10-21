// Parameters Standardized Document Definition
package signed_docs

_parameters_description: """
	{{ .doc }} Parameters define the parameter data required for the
	system at the {{ .doc }} level.

	Parameter Data includes things such as:

	* Functional parameters
	* Timeline data
	* Branded Content and Copy

	The content of the parameters is defined solely by the 
	{{ .doc }} Parameters Form Template.

	This allows parameters to vary based on individual system
	requirements over time.

	Functional Parameters are mapped using the (TBD Functional Parameters Map).

	The payload of a {{ .doc }} is controlled by its template.
	"""

_parameters_parent_validation: """
	The {{ .doc }} Parameters Document *MUST* be linked through `parameters` to 
	its {{ .doc_parent }} Parameters Document.
	"""

_parameters_payload_description: """
	{{ .doc }} Parameters Document controlling the {{ .doc }} 
	within a {{ .doc_parent }}.

	Must be valid according to the schema contained within the 
	`Document Reference` from the `template` metadata.
	"""

#generic_parameters: #signedDocument & {

	description: _

	metadata: {
		template: required: "yes"

		collaborators: required: "optional"

		revocations: required: "optional"

		parameters: required: _ | *"yes"
	}

	headers: "content type": value: "application/json"

	payload: description: _

	signers: {
		roles: {
			user: []
			admin: [
				"Brand Admin",
			]
		}
		update: collaborators: "collaborators"
	}

	authors: {
		"Steven Johnson": "steven.johnson@iohk.io"
		"Nathan Bogale":  "nathan.bogale@iohk.io"
	}

}

_generic_parameters_versions: [
	{
		version:  "0.01"
		modified: "2025-04-04"
		changes: """
			* First Published Version
			"""
	},
	{
		version:  "0.02"
		modified: "2025-06-20"
		changes: """
			* Generalized as another kind of form data document
			"""
	},
]
