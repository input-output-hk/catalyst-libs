package signed_docs

// Proposal Document Definition

docs: #DocumentDefinitions & {
	"Proposal Comment": {
		description: """
			A Proposal Comment is a document which comments on a referenced Proposal document.
			
			Proposal Comments themselves are intentionally general, however they may be
			linked to a brand/campaign or category via the template used by the proposal.

			The payload of a proposal comment is controlled by its template.
			"""
		// The description of this document.  Markdown Supported.
		validation: """
			A comment which is a reply *MUST* reference the same document.
			It may reference a different version of the document.
			"""

		// The business logic related to this document.  Markdown Supported.
		business_logic: {
			front_end: """
				Comments are valid for any version of the document, however
				as comments refer to a specific version of a document, they may
				lose context when displayed against the latest version of a document.
				In these cases, the front end should clearly show that a comment was on
				a different version of the document.

				If the front end posts a reply to another comment: 

				* it should reference the comment being replied to in the `reply` field. 
				* The `ref` field must refer to the same document, but can be a different version.
				"""

			back_end: """
				The backend will only validate the document being referenced exists, 
				and the integrity of the `ref` and `reply` metadata fields is correct.
				"""
		}
		metadata: {
			ref: {
				required: "yes"
				type:     "Proposal"
			}

			reply: {
				required: "optional"
				type:     "Proposal Comment"
			}

			section: required: "optional"

			template: {
				required: "yes"
				type:     "Proposal Comment Template"
			}

			revocations: required: "optional"

			parameters: {
				required: "yes"
				type:     doc_clusters."System Parameters".docs
				linked_refs: [
					"ref",
					"template",
				]
			}
		}

		payload: description: """
			JSON Document which must validate against the referenced template.
			"""

		versions: [
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
		]
	}
}
