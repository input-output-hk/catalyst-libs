// Template Json Schema Definitions Dictionary
// 
// All known and supported Json Schema definitions,
// and their parameters and documentation so that
// a dictionary document and the definitions themselves
// can be generated.
package template_field_definitions

// Types of a Metadata Fields
#templateJsonSchemaDefNames: or([
	for k, _ in dictionary {k},
])

// Definitions for all defined template schema field types.
dictionary: #jsonSchemaFields & {}

// Types of a Metadata Fields
#templateJsonSchemaDefNames: or([
	for k, _ in dictionary {k},
])
