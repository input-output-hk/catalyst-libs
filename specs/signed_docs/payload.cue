package signed_docs

// Payload definition
_payload: {
	// Description of the payload
	description: string
	// Optional fixed schema for the payload.
	// A URI or inline JSON Schema that the payload must validate against.
	schema?: _
}
