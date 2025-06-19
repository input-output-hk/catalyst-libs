@extern(embed)

package signed_docs

docs: #DocumentDefinitions & {
	"Voter Representative Delegation": {
		description: """
			  ## Voter Representative Delegation Document

			  Captures that a voter (the signer) has delegated to a Representative for a specific category.
			  The document this refers to (`ref`) is the Representative's Category Profile.
			  The category itself is specified in the `parameters` metadata.
			"""
		validation: """
			  The payload must contain a 'status' field, which must be either 'active' or 'revoked'.
			  The Category id for the Representative's Category Profile and as specified in the metadata must match. 
			"""
		business_logic: {
			front_end: """
				  Allow voters to delegate to a Representative for a category ('active') or revoke that delegation ('revoked').
				"""
			back_end: """
				  Validate the delegation action and update the voter's delegation state for the given category and Representative.
				"""
		}
		metadata: {
			ref: {
				required: "yes"
				type:     "Representative Category Profile"
			}
			parameters: {
				required: "yes"
				type:     "Category Parameters"
			}
		}
		payload: {
			description: """
				  A minimal payload indicating the intended status of the delegation.
				  'active' creates or affirms the delegation.
				  'revoked' withdraws the delegation.
				"""
			schema: _ @embed(file="payload_schemas/voter_representative_delegation.schema.json")
		}
		signers: {
			roles: {
				user: [
					"Registered",
				]
			}
		}
		authors: {
			"Neil McAuliffe": "neil.mcauliffe@iohk.io"
		}
		versions: [
			{
				version:  "0.01"
				modified: "2025-06-19"
				changes: """
					* First Published Version
					"""
			},
		]
	}
}
