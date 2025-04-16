@extern(embed)

package signed_docs

// Proposal Submission Action
docs: #DocumentDefinitions & {
	"Proposal Submission Action": {
		description: """
			## Proposal Submission Action

			A Proposal Submission Action is a document which can attempt to either submit a 
			particular version of a proposal into a campaign, or withdraw it.

			The last action on the document ts the action which takes effect at the deadline.

			For multiple collaborators, multiple submission actions can be posted independently, 
			but none of them will take effect until ALL collaborators have posted equivalent actions.

			For example, three collaborators Alice/Bob/Claire can each post one submission action
			for the same document.  
			Unless they all submit the same version of the proposal
			the proposal will not be seen as submitted.

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
				they will be considered as agreeing to be a collaborator on any version of the document
				that lists them, if their latest submission is `draft` or `final`.

				If their latest submission on a document is `hide` they should be considered to not
				have agreed to be a collaborator.

				*NOTE* `final` status ONLY applies to the exactly referenced document and version.
				"""

			back_end: """
				A Submitted proposal with collaborators *MUST* have a `final` submission by *ALL* listed `collaborators`.
				If any `collaborator` has not submitted a `final` submission by the deadline, then the proposal 
				is not considered `final` and will not be considered in the category it was being submitted to.
				"""
		}
		metadata: {
			ref: {
				type:     "Proposal"
				required: "yes"
				multiple: true
			}

			category_id: {
				required: "yes"
				type:     "Category Parameters"
			}
		}

		payload: {
			description: """
				The kind of action is controlled by this payload.
				The Payload is a JSON Document, and must conform to this schema.

				States:

				* `final` : All collaborators must publish a `final` status for the proposal to be `final`.
				* `draft` : Reverses the previous `final` state for a signer and accepts collaborator status to a document.  
				* `hide`  : Requests the proposal be hidden (not final, but a hidden draft).  
							`hide` is only actioned if sent by the author, 
							for a collaborator it identified that they do not wish to be listed as a `collaborator`.
				"""
			schema: _ @embed(file="payload_schemas/proposal_submission_action.schema.json")
		}

		"signers": {
			roles: {
				// Proposers may publish this document.
				user: [
					"Proposer",
				]
			}

			referenced: true

			update: {
				collaborators: true
			}
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
