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
_payload: {
	// Description of the payload
	description: string
	// Is the Payload nil?
	nil: bool | *false

	// Only have these when the payload isn't nil.
	if !nil {
		// Optional fixed schema for the payload.
		// A URI or inline JSON Schema that the payload must validate against.
		schema?: _
		// Examples of the schema.
		examples?: list.UniqueItems
		examples?: [...#payloadExample] | *[]
	}
}
