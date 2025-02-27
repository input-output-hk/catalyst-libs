package signed_docs

#SignedDoc: {
	// Signed Document Definitions Database
	//
	// Source of truth for definitions of all catalyst signed
	// documents.
	@jsonschema(schema="http://json-schema.org/draft-07/schema#")
	null | bool | number | string | [...] | close({
		@jsonschema(id="https://raw.githubusercontent.com/input-output-hk/catalyst-libs/refs/heads/main/docs/src/architecture/08_concepts/signed_doc/signed_doc_defs.schema.json")
		default?: #docDefinitionFields

		// Document Definition
		docs?: {
			{[=~".*"]: #docDefinition}
			"Proposal Template"!:         _
			Proposal!:                    _
			"Proposal Action"!:           _
			"Proposal Comment Template"!: _
			"Proposal Comment"!:          _
			"Category Parameters"!:       _
			"Campaign Parameters"!:       _
			"Brand Parameters"!:          _
			"Public Vote Tx V2"!:         _
			"Private Vote Tx V2"!:        _
			"Immutable Ledger Block"!:    _
			...
		}
	})

	#contentType: "application/json" | "application/cbor"

	#contentEncoding: "br"

	#contentEncodings: [...#contentEncoding]

	#docType: =~"^[0-9A-Fa-f]{8}-[0-9A-Fa-f]{4}-4[0-9A-Fa-f]{3}-[89ABab][0-9A-Fa-f]{3}-[0-9A-Fa-f]{12}$"

	#docIdOrVer: =~"^[0-9A-Fa-f]{8}-[0-9A-Fa-f]{7}-4[0-9A-Fa-f]{3}-[89ABab][0-9A-Fa-f]{3}-[0-9A-Fa-f]{12}$"

	#metadataFormat: "UUIDv7" | "COSE Algorithm" | "HTTP Content Type" | "HTTP Content Encoding" | "Document Reference" | "Document Hash"

	#requiredOrOptional: "yes" | "optional" | "no"

	#metadataFieldDescription: string

	#metadataFieldSpec: close({
		format?:      #metadataFormat
		required?:    #requiredOrOptional
		"ref type"?:  string
		description?: #metadataFieldDescription
		validation?:  #metadataFieldDescription
	})

	#docDefinitionFields: null | bool | number | string | [...] | {
		contentType?:      #contentType
		contentEncodings?: #contentEncodings
		metadata?: close({
			id?:                 #metadataFieldSpec
			ver?:                #metadataFieldSpec
			alg?:                #metadataFieldSpec
			"content type"?:     #metadataFieldSpec
			"content encoding"?: #metadataFieldSpec
			ref?:                #metadataFieldSpec
			"ref hash"?:         #metadataFieldSpec
			template?:           #metadataFieldSpec
		})
		...
	}

	#docDefinition: {
		@jsonschema(id="https://raw.githubusercontent.com/input-output-hk/catalyst-libs/refs/heads/main/docs/src/architecture/08_concepts/signed_doc/docDefinition")
		...
	}
}
