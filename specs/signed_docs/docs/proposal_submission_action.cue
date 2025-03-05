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
			for the same document.  Unless they all submit or withdraw the same version of the proposal
			the proposal will not be seen as submitted or withdrawn.

			The payload is a fixed format.
			"""

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
				* `draft` : Reverses the previous `final` state for a signer.  
				* `hide`  : Requests the proposal be hidden (not final, but a hidden draft).  
							`hide` is only actioned if sent by the author, for a collaborator its synonymous with `draft`.
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

	}
}
