package examples

import (
	"list"
)

// Individual Payload Example
#item: {
	// Title of the example
	title: string
	// Expanded description of what the example shows.
	description: string
	// Example data value.
	example: _
}

// A List of examples. (each must be unique)
#list: list.UniqueItems
#list: [...#item] | *[]
