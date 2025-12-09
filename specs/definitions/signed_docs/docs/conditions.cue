package signed_docs

import (
	"github.com/input-output-hk/catalyst-libs/specs/signed_doc_types"
)

// Conditions Document Definition

docs: #DocumentDefinitions & {
	Conditions: {
		description: """
			Conditions documents define terms and conditions that users must accept
			before submitting documents to the system.
			
			Supports multiple condition types (TOU, license agreements, operational
			guidelines, regional restrictions).
			
			The payload of a Conditions document contains the text of the terms and
			conditions, typically in Markdown or HTML format.
			This allows for rich formatting while maintaining human readability.
			
			Conditions documents are versioned and can be revoked, enabling
			administrators to update terms over time while maintaining an auditable
			history of what terms were in effect at any given time.
			"""

		validation: """
			The Conditions document *MUST* be a valid signed document according to
			the Signed Document Standard.
			
			When a Conditions document is referenced in a parameter document's
			`conditions` metadata field, the referenced document *MUST* exist and be
			of type "Conditions".
			
			When a Conditions document is referenced in a user-submitted document's
			`conditions` metadata field, the referenced document *MUST* exist, be of
			type "Conditions", and not be revoked.
			"""

		business_logic: {
			front_end: """
				Front-end applications should:
				
				* Display Conditions documents to users when they are required to
				  accept them
				* Store user acceptance locally to minimize friction (users only need
				  to explicitly accept conditions the first time they encounter them)
				* Gray out submission buttons until all required conditions have been
				  accepted
				* Display a disclosure on submission listing all accepted conditions
				  under which the document is being submitted
				* Provide clear error messages if required conditions are missing or
				  invalid
				"""

			back_end: """
				Back-end validation must:
				
				* Verify that all Conditions documents referenced in user-submitted
				  documents exist and are valid
				* Collect all required conditions from the parameter hierarchy
				  (Brand → Campaign → Category → Contest)
				* Ensure user-submitted documents include exactly the union of all
				  required conditions from their parameter hierarchy
				* Reject documents that reference revoked Conditions documents
				* Reject documents that are missing required conditions or include
				  conditions not in the parameter hierarchy
				
				The decentralized system (Hermes) will also reject documents without
				the required conditions, ensuring validation occurs at multiple layers.
				"""
		}

		headers: "content type": value: [
			"text/markdown; charset=utf-8",
			"text/html; charset=utf-8",
		]

		headers: "content-encoding": value: ["br"]

		metadata: {
			ref: {
				required: "optional"
				type:     signed_doc_types.allDocNames
			}

			collaborators: {
				required: "optional"
				description: """
					A list of collaborators who may be associated with this document.

					**Important**: For Conditions documents, only the original author can update
					and sign new versions. Collaborators listed here do not have permission to
					publish updates to this document. This field is optional and may be used for
					documentation or organizational purposes only.
				"""
				validation: """
					For Conditions documents, collaborators do not have update permissions.
					Only the original author can create new versions of Conditions documents.

					In the event there are **MULTIPLE** `collaborators` listed, they **MUST** be
					sorted.

					Sorting for each element of `collaborators` follows the same sort order as
					specified for Map Keys, as defined by CBOR Deterministic Encoding
					(4.3.2 Length-First Map Key Ordering).
				"""
			}

			revocations: required: "optional"
		}

		payload: description: """
			The Conditions document payload contains the text of the terms and
			conditions.
			
			The payload *MUST* be valid according to the content type specified in
			the COSE header:
			
			* If `content-type` is `text/markdown; charset=utf-8`, the payload must be
			  valid Markdown
			* If `content-type` is `text/html; charset=utf-8`, the payload must be
			  valid HTML5
			
			The payload is compressed using Brotli compression (`br` encoding) as
			specified in the `content-encoding` header.
			
			The payload content should be human-readable and clearly state:
			* The purpose of the conditions
			* What users are agreeing to
			* Any obligations or restrictions
			* Effective dates or version information
			* Contact information for questions
			"""

		signers: {
			roles: admin: [
				"Brand Admin",
				"Campaign Admin",
				"Category Admin",
				"Contest Admin",
			]

			update: type: "author"
		}

		authors: {
			"Nathan Bogale":  "nathan.bogale@iohk.io"
			"Steven Johnson": "steven.johnson@iohk.io"
		}

		versions: [
			{
				version:  "0.01"
				modified: "2025-01-XX"
				changes: """
					* First Published Version (DRAFT)
					"""
			},
		]
	}
}

