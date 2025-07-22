// Regex Definitions
package regex

_camelCase: #"[a-z]+((\d)|([A-Z0-9][a-z0-9]+))*([A-Z])?"#

_camelCaseName: "^\(_camelCase)$"

def: #def & {
	singleLine: {
		pattern:     #"^[^\n]*$"#
		description: "Single Line of Text"
	}
	multiLine: {
		pattern:     #"^[\S\s]*$"#
		description: "Multiline Text"
	}
	httpsUrl: {
		pattern:     #"^https://[^\s]+$"#
		description: "A URL that must begin with https://"
	}
	iconName: {
		pattern:     #"^[a-z0-9]+(-[a-z0-9]+)*$"#
		description: "An Icon name can only have lower case letters or numbers and a single dash between them."
	}
	jsonSchemaDef: {
		pattern: "^#/\\$defs/(\(_camelCase))$"
		description: """
			A reference to a json schema definition in the same file.
			Captures the definitions name as well.
			"""
	}
	camelCaseName: {
		pattern: _camelCaseName
		description: """
			A name that must conform to `camelCase`.
			"""
	}
}

// Every definition above MUST have at least one test below
// for the positive and negative match.
positive_match: true

positive_match: "" =~ def.singleLine.pattern
positive_match: "single line" =~ def.singleLine.pattern

positive_match: "single line" =~ def.multiLine.pattern
positive_match: "multi\nline" =~ def.multiLine.pattern
positive_match: "" =~ def.multiLine.pattern

positive_match: "https://www.iana.org/assignments" =~ def.httpsUrl.pattern

positive_match: "an-icon" =~ def.iconName.pattern
positive_match: "icon" =~ def.iconName.pattern
positive_match: "an-icon-with-many-parts-1" =~ def.iconName.pattern

positive_match: "#/$defs/a" =~ def.jsonSchemaDef.pattern
positive_match: "#/$defs/aValue" =~ def.jsonSchemaDef.pattern
positive_match: "#/$defs/a123" =~ def.jsonSchemaDef.pattern
positive_match: "#/$defs/aValue123" =~ def.jsonSchemaDef.pattern

positive_match: "a" =~ def.camelCaseName.pattern
positive_match: "aValue" =~ def.camelCaseName.pattern
positive_match: "a123" =~ def.camelCaseName.pattern
positive_match: "aValue123" =~ def.camelCaseName.pattern

// Negative match (where possible to test)
negative_match: false

negative_match: "multi\nline" =~ def.singleLine.pattern

// No negative multiline regex cases. Regex too simple.

negative_match: "" =~ def.httpsUrl.pattern
negative_match: "not a url" =~ def.httpsUrl.pattern
negative_match: "http://www.iana.org/assignments" =~ def.httpsUrl.pattern

negative_match: "an_icon" =~ def.iconName.pattern
negative_match: "-icon" =~ def.iconName.pattern
negative_match: "an-Icon-with-many-parts-1" =~ def.iconName.pattern
negative_match: "an-icon--with-many-parts-1" =~ def.iconName.pattern

negative_match: "#/$defs/" =~ def.jsonSchemaDef.pattern
negative_match: "#/$defs/a Value" =~ def.jsonSchemaDef.pattern
negative_match: "Just very wrong" =~ def.jsonSchemaDef.pattern
negative_match: "#/$defs/123" =~ def.jsonSchemaDef.pattern
negative_match: "#/$defs/123aValue" =~ def.jsonSchemaDef.pattern

negative_match: "" =~ def.camelCaseName.pattern
negative_match: "a Value" =~ def.camelCaseName.pattern
negative_match: "Just very wrong" =~ def.camelCaseName.pattern
negative_match: "123" =~ def.camelCaseName.pattern
negative_match: "123aValue" =~ def.camelCaseName.pattern
