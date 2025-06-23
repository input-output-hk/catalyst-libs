// Signed Document Definitions
// 
// Metadata Types and Constraints
@extern(embed)
package signed_docs

_chainValidMermaid:      _ @embed(file=includes/valid_chain.mermaid,type=text)
_chainInvalidMermaid:    _ @embed(file=includes/invalid_chain.mermaid,type=text)
_chainFraudulentMermaid: _ @embed(file=includes/fraudulent_chain.mermaid,type=text)

_chain_validation_common: """
	Chained Documents do not support collaborators.
	Any document which is attempted to be published in the sequence
	which is *NOT* published by the author of the first document in the
	sequence is fraudulent, and to be discarded.

	In addition, the chained document *MUST*:

	* Not have `collaborators`;
	* Have the same `id` as the document being chained to;
	* Have a `ver` that is greater than the `ver` being chained to;
	* Have the same `type` as the chained document;
	* Have `parameters` match;
	* Have not be chaining to a document already chained to by another document;
	* Have its absolute `height` exactly one more than the `height` of the document being chained to.

	IF any of these validations fail, then the entire sequence of documents is INVALID.
	Not just the current document.
	"""

_chain_validation_examples: """
	##### Example of a Valid Chain

	<!-- markdownlint-disable MD046 -->
	``` mermaid
	\(_chainValidMermaid)
	```
	<!-- markdownlint-enable MD046 -->

	##### Example of an Invalid Chain

	Either of the two documents being present invalidates the data
	in the entire chain,
	as they are signed by the author of the chain.

	<!-- markdownlint-disable MD046 -->
	``` mermaid
	\(_chainInvalidMermaid)
	```
	<!-- markdownlint-enable MD046 -->

	##### Example of a Fraudulent Chain Document

	The invalid document does not invalidate the chain,
	as its not signed by the author of the chained documents.

	<!-- markdownlint-disable MD046 -->
	``` mermaid
	\(_chainFraudulentMermaid)
	```
	<!-- markdownlint-enable MD046 -->
	"""

_chain_validation_complete: """
	\(_chain_validation_common)

	\(_chain_validation_examples)
	"""

#metadata: chain: {
	format: "Chain Link"
	description: """
		An immutable link to the previous document in a chained sequence of documents.
		Because ID/Ver only defines values for the current document, and is not intended 
		by itself to prevent insertion of documents in a sequence, the `chain`
		metadata allows for the latest document to directly point to its previous iteration.

		It also aids in discoverability, where the latest document may be pinned but prior
		documents can be discovered automatically by following the chain.
		"""
	validation: string | *"""
			\(_chain_validation_common)
			"""
}

metadata: headers: chain: {
	required:   "optional"
	validation: _chain_validation_complete
}
