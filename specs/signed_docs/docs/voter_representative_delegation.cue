@extern(embed)

package signed_docs

docs: #DocumentDefinitions & {
	"Voter Representative Delegation": {
		description: "## Voter Representative Delegation\nA signed document that allows a 'Voter' to delegate their voting power to a 'Representative' for a specific category."
		validation: """
			* The signer MUST be a registered 'Voter'.
			* The 'ref' metadata field MUST point to a valid 'Representative Category Profile'.
			* The payload MUST be empty.
			"""
		business_logic: {
			front_end: """
				* Allows a voter to select a Representative from a list of eligible candidates for a category.
				* The voter signs this document to confirm their delegation choice.
				"""
			back_end: """
				* Verifies that the voter and Representative are valid and registered for the category.
				* Records the delegation of voting power from the voter to the Representative.
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
