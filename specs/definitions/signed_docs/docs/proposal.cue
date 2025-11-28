package signed_docs

import (
	"github.com/input-output-hk/catalyst-libs/specs/signed_doc_types"
)

// Proposal Document Definition

docs: #DocumentDefinitions & {
	Proposal: {
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

			All versions of the document are owned by the original author.
			The Author can not be changed by any document revision.

			Any Proposal that lists a collaborator is an invitation for that collaborator to participate in the proposal.
			They are considered to have accepted that invitation for **all** versions of the proposal that
			list them as a collaborator where their latest
			[Proposal Submission Action](proposal_submission_action.md) for that proposal has an `action` of
			`draft` or `final`.

			If a collaborator’s latest [Proposal Submission Action](proposal_submission_action.md) for the
			proposal has an `action` of `hide`, they **MUST** be treated as not having agreed to collaborate
			for **any** version of that proposal (past, present, or future) until they later submit `draft`
			or `final` again.

			The requirement for collaborator submissions when finalizing a proposal is controlled by a
			Brand/Campaign/Category parameter (name TBD). When configured for unanimous collaboration,
			every collaborator listed on the submitted version **MUST** also publish a `final`
			[Proposal Submission Action](proposal_submission_action.md) alongside the author.
			When configured for opt-in collaboration (the default, and the behavior when the parameter is
			absent), only collaborators who submit `final` for the referenced version are included as
			collaborators on that submission; collaborators who do not submit `final` are not treated as
			collaborators for that submission.
			In all cases, a proposal cannot be final unless the original author has submitted `final`.

			The `final` proposal itself may be signed by one or more Collaborators and/or the original Author.
			The `final` proposal must never be signed by anyone else.
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
		headers: "content type": value: "application/json"

		metadata: {
			template: {
				required: "yes"
				type:     "Proposal Form Template"
			}

			collaborators: required: "optional"

			revocations: required: "optional"

			parameters: {
				required: "yes"
				type:     signed_doc_types.doc_clusters."System Parameters".docs
				linked_refs: [
					"template",
				]
			}
		}

		payload: description: """
			Proposal Document drafted for submission to a category of a campaign.

			Must be valid according to the schema contained within the 
			`Document Reference` from the `template` metadata.
			"""

		signers: {
			roles: user: [
				"Proposer",
			]

			update: type: "collaborators"
		}

		authors: "Steven Johnson": "steven.johnson@iohk.io"

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
