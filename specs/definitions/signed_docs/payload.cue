package signed_docs

import (
	"list"
)

// Individual Payload Example
#payloadExample: {
	// Title of the example
	title: string
	// Expanded description of what the example shows.
	description: string
	// Example data that matches the payload schema.
	example: _
}

// Payload definition
#payload: {
	// Description of the payload
	description: string

	// Is the Payload allowed to be nil?
	nil: true | *false
}


// Payload definition
#payload_json: #payload & {
	// Optional fixed schema for the payload.
	// A URI or inline JSON Schema that the payload must validate against.
	schema?: string
	// Examples of the schema.
	examples?: list.UniqueItems
	examples?: [...#payloadExample] | *[]
}

// Payload definition for cbor payloads
#payload_cbor: #payload & {
	// CBOR payloads must have a CDDL Schema defined.
	schema?: #cddlTypesConstraint
	
	// Examples of the schema.
	examples?: list.UniqueItems
	examples?: [...#payloadExample] | *[]
}
