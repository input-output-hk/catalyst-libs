package signed_docs

// Proposal Document Definition

docs: #DocumentDefinitions & {
	"Proposal": {
		description: """
			A Proposal is a document which describes a proposed solution or project to
			address the criteria of a category within a campaign.
			
			The proposal itself is a draft document, it is not submitted for consideration
			unless a `Proposal Submission Action` is submitted which references it.

			Proposals themselves are intentionally general, however they may be
			linked to a brand/campaign or category via the template used by the proposal.

			The payload of a proposal is controlled by its template.
			"""

		validation: """
			The first version of a Proposal *MUST* be signed by the original author.
			It may optionally be co-signed by any of the listed `collaborators`.
			It may not be signed by anyone else.

			Subsequent Versions can be signed/co-signed by either the Original Author of the first version,
			OR any of the listed `collaborators` in the immediately previous version.
			This allows any collaborator to update the next version of a document, provided they are still a collaborator.
			It is valid for a proposal to be signed by a collaborator 
			who is no longer listed as in the `collaborators`
			of the document they are signing, provided they are listed as a collaborator in the immediately previous document version.
			This allows for a collaborator to make an update to the document which removes themselves
			from the `collaborators` list.

			All versions of the document *MUST* list the author as the original author.
			The Author can not be changed by any document revision.
			"""

		business_logic: {

			front_end: """
				As validity of the documents is currently enforced by the backend, 
				the front end does not need to validate the document has been signed
				correctly.
				It may do so, but it is not required.
				"""

			back_end: """
				Before accepting a new proposal to be published, the backend will ensure:

				* The document has been signed by a valid author or collaborator.
				* That the signer of the document was a registered proposer
				* That the document was signed with their proposers key
				* That all listed `collaborators` are registered as proposers.
				* That the document has been signed validly according to the [validation](#validation) rules.
				"""
		}

		metadata: {
			template: {
				required: "yes"
				type:     "Proposal Template"
			}

			collaborators: {
				required: "optional"
			}

			revocations: {
				required: "optional"
			}

			brand_id: {
				required: "optional"
				type:     "Brand Parameters"
				linked_refs: [
					"template",
				]
			}
			campaign_id: {
				required: "optional"
				type:     "Campaign Parameters"
				linked_refs: [
					"template",
				]
			}
			category_id: {
				required: "optional"
				type:     "Category Parameters"
				linked_refs: [
					"template",
				]
			}
		}

		payload: {
			description: """
				Proposal Document drafted for submission to a category of a campaign.

				Must be valid according to the schema contained within the 
				`Document Reference` from the `template` metadata.
				"""
		}

		signers: {
			roles: {
				user: [
					"Proposer",
				]
			}

			update: {
				"collaborators": true
			}
		}

		authors: {
			"Steven Johnson": "steven.johnson@iohk.io"
		}

		versions: [
			{
				version:  "0.01"
				modified: "2025-04-04"
				changes: """
					* First Published Version
					"""
			},
		]
	}
}
