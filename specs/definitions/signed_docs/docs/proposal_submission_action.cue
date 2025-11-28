@extern(embed)

package signed_docs

import (
	"github.com/input-output-hk/catalyst-libs/specs/signed_doc_types"
)

// Proposal Submission Action
docs: #DocumentDefinitions & {
	"Proposal Submission Action": {
		description: """
			Proposal Submission Action

			A Proposal Submission Action is a document which can attempt to either submit a 
			particular version of a proposal into a campaign, or withdraw it.

			The last action on the document is the action which takes effect at the deadline.

			For multiple collaborators, multiple submission actions can be posted independently.
			How those submissions are counted is controlled by a parameter defined in the relevant
			Brand/Campaign/Category parameters (parameter name TBD):

			* If configured for unanimous collaboration, actions for a proposal version do not take
			  effect until the author and **all** listed collaborators have posted equivalent actions.
			* If configured for opt-in collaboration (the default, and the behavior when the parameter
			  is absent), the author must submit `final`, and collaborators are only counted on the
			  submission when they submit `final` for that version; other listed collaborators are not
			  required to post equivalent actions.

			For example, three collaborators Alice/Bob/Claire can each post one submission action
			for the same document.  
			Under the unanimous configuration, unless they all submit the same version of the proposal
			the proposal will not be seen as submitted.
			Under the opt-in configuration, the author must submit `final` and whichever collaborators
			also submit `final` for that version are counted as collaborators on that submission.

			The required set of signers for a submitted proposal version is:

			* The original author of the proposal (the signer of the first version where
			  [`id`](../metadata.md#id) == [`ver`](../metadata.md#ver)); and
			* A collaborator set derived from the configuration described above.

			When collaborator unanimity is configured, that set is every collaborator listed in
			[`collaborators`](../metadata.md#collaborators) on the **exact** version of the proposal
			referenced by [`ref`](../metadata.md#ref).
			When opt-in is configured (the default/fallback), only collaborators who submit `final`
			for the referenced version are included as collaborators on that submission.
			In all modes, a proposal is only final if the author has submitted `final`.

			They may jointly sign a single Proposal Submission Action, or may submit multiple independent
			Submission actions for the same document (which avoids the need for multi-sig coordination).

			The payload is a fixed format.
			"""
		validation: """
			No validation is required beyond as defined by:
			
			* [metadata](#metadata) 
			* [payload](#payload)
			* [signers](#signers)
			"""

		// The business logic related to this document.  Markdown Supported.
		business_logic: {
			front_end: """
				A proposal with `collaborators` will not be shown as having a confirmed collaborator,
				unless there exists a `draft` or `final` proposal submission from that collaborator.

				Any document that lists a collaborator should be highlighted to that collaborator so
				they can take appropriate action, such as:

				* Confirm they are a collaborator by submitting this document as `draft`
				* Agree to being a collaborator on the final submission by submitting this document as `final`
				* Hide themselves from the collaborators list but do not remove themselves by submitting `hide`
				* Remove themselves permanently as a collaborator by publishing a new version with them removed.

				To eliminate the necessity for collaborators to accept collaboration on every version, 
				they will be considered as agreeing to be a collaborator on **any** version of the proposal that
				lists them in [`collaborators`](../metadata.md#collaborators), if their latest submission for that
				proposal is `draft` or `final`.

				If their latest submission on a proposal is `hide` they **MUST** be considered to not have agreed
				to be a collaborator for **any** version of that proposal (past, present, or future) until they
				submit a new `draft` or `final` action.

				Whether collaborator submissions are required for finalization follows the
				Brand/Campaign/Category parameter described above.
				If the parameter is absent, only collaborators who submit `final` for the referenced version are
				counted as collaborators on that submission; listed collaborators who do not submit `final` are
				not required for the proposal to be considered final.

				*NOTE* `final` status ONLY applies to the exactly referenced document and version.
				"""

			back_end: """
				A Submitted proposal with collaborators *MUST* have, by the configured deadline:

				* A `final` submission from the author; and
				* Collaborator submissions as required by the Brand/Campaign/Category parameter that controls
				  collaborator finalization (parameter name TBD):
					* If set to unanimous, a `final` submission from every collaborator listed in
					  [`collaborators`](../metadata.md#collaborators) on the version of the proposal 
					  referenced by [`ref`](../metadata.md#ref); or
					* If set to opt-in (the default, and the behavior when the parameter is absent), only
					  collaborators whose latest submission for that proposal version is `final` are included as
					  collaborators on the submission.
				* No required signer (author or any collaborator counted under the configured mode) whose latest
				  submission for that proposal is `hide`.

				If the author has not submitted a `final` submission for that proposal version by the deadline,
				or if any collaborator required by the configured mode has not submitted a `final` submission, or
				if any required signer’s latest submission is `hide`, 
				then the proposal is not considered `final` and will not be considered in the 
				category it was being submitted to.
				"""
		}

		headers: "content type": value: "application/json"

		metadata: {
			ref: {
				type:     "Proposal"
				required: "yes"
			}

			parameters: {
				required: "yes"
				type:     signed_doc_types.doc_clusters."System Parameters".docs
				linked_refs: [
					"ref",
				]
			}
		}

		payload: {
			description: """
				The kind of action is controlled by this payload.
				The Payload is a JSON Document, and must conform to this schema.

				States:

				* `final` : Signer marks this proposal version as their final submission.
				            Whether all collaborators must also submit `final` is controlled by the
				            Brand/Campaign/Category parameter noted above.
				            If the parameter is absent, only collaborators who submit `final` for the referenced
				            version are counted as collaborators on the submission, but the author’s `final` is
				            always required.
				* `draft` : Reverses the previous `final` state for a signer and accepts collaborator status to a document.  
				* `hide`  : Requests the proposal be hidden (not final, but a hidden draft).  
							`hide` is only actioned if sent by the author, 
							for a collaborator it identified that they do not wish to be listed as a `collaborator`.
				"""
			schema: _ @embed(file="payload_schemas/proposal_submission_action.schema.json")
			examples: [
				{
					title: "Final Proposal Submission"
					description: """
						This document indicates the linked proposal is final and requested to proceed for further consideration.
						"""
					example: _ @embed(file="payload_schemas/proposal_submission_action.final.example.json")
				},
				{
					title: "Draft Proposal Submission"
					description: """
						This document indicates the linked proposal is no longer final and should not proceed for further consideration.
						It is also used by collaborators to accept that they are a collaborator on a document.
						"""
					example: _ @embed(file="payload_schemas/proposal_submission_action.draft.example.json")
				},
				{
					title: "Hidden Proposal Submission"
					description: """
						If submitted by the proposal author the document is hidden, it is still public but not shown as
						a proposal being drafted.
						If submitted by a collaborator, that collaborator is declaring they do not wish to be listed as
						a collaborator on the proposal.
						"""
					example: _ @embed(file="payload_schemas/proposal_submission_action.hide.example.json")
				},
			]
		}
		signers: {
			roles: {
				// Proposers may publish this document.
				user: [
					"Proposer",
				]
			}

			update: type: "ref"
		}

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
