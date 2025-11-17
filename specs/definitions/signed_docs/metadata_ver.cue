// Signed Document Definitions
// 
// Metadata Types and Constraints
// `ver`
@extern(embed)
package signed_docs

_ver_description_common: """
	The unique version of the document.
	The first version of the document must set `ver` == `id`
	"""

_ver_description_complete: """
	\(_ver_description_common)

	`ver` represents either:

	* when a document changes over time, such as
		with a new version of a particular document that supersedes an 
		earlier one.
	* when a new document in a sequence of documents is produced.
		
	Because the most common use `ver` is a new version of the same document
	this is to be assumed unless the document specifies its representing
	a sequence of documents.
	"""

_ver_description_versioned: """
	\(_ver_description_common)

	`ver` represents new versions of the same document as it changes over time.
	"""

_ver_description_sequenced: """
	\(_ver_description_common)

	`ver` represents new documents in a sequence of documents.
	"""

// Document Version
#metadata: {
	ver: {
		required:    "yes"
		format:      "Document Ver"
		description: string | *_ver_description_versioned
		validation: """
			1. The document version must always be >= the document ID.
			2. IF `ver` does not == `id` then a document with `id` and `ver` being equal *MUST* exist.
			3. When a document with the same `id` already exists, the new document's `ver` must be greater than
    the latest known submitted version for that `id`.
			4. When a document with the same `id` already exists, the new document's `type` must be the same as
    the latest known submitted document's `type` for that `id`.
			"""
	}
}

// Note: we make all normally excluded fields optional at the global level, because they are globally optional
metadata: headers: ver: description: _ver_description_complete
