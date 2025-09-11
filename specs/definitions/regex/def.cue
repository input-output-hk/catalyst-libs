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
	cardName: {
		pattern: #"^[a-z][a-z0-9]*([-/][a-z0-9]+)*$"#
		description: """
			The format thats enforces how presentation template card names will be formatted.
			"""
	}
	titleCaseName: {
		pattern: #"^[A-Z][a-z]*([ ][A-Z][a-z]*)*$"#
		description: """
			A name where every word starts with a capital letter.
			"""
	}
	jsonContentType: {
		pattern: #"^application\/(?:json|[a-z0-9!#$&^_.+-]+\+json)(?:\s*;\s*[^=]+=[^;]+)*$"#
		description: """
			Matches any known json content type.
			"""
	}
	cborContentType: {
		pattern: #"^application\/(?:cbor|[a-z0-9!#$&^_.+-]+\+cbor)(?:\s*;\s*[^=]+=[^;]+)*$"#
		description: """
			Matches any known cbor content type.
			"""
	}
	cddlContentType: {
		pattern: #"^application\/(?:cddl|[a-z0-9!#$&^_.+-]+\+cddl)(?:\s*;\s*[^=]+=[^;]+)*$"#
		description: """
			Matches any known cddl content type.
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

positive_match: "a" =~ def.cardName.pattern
positive_match: "a-value" =~ def.cardName.pattern
positive_match: "a-namespace/a-value" =~ def.cardName.pattern
positive_match: "a-namespace/sub-namespace/a-new-name" =~ def.cardName.pattern

positive_match: "A" =~ def.titleCaseName.pattern
positive_match: "A Title" =~ def.titleCaseName.pattern
positive_match: "A Title Case" =~ def.titleCaseName.pattern
positive_match: "A Title Case Name" =~ def.titleCaseName.pattern

positive_match: "application/json" =~ def.jsonContentType.pattern
positive_match: "application/json; charset=UTF-8" =~ def.jsonContentType.pattern
positive_match: "application/schema+json; profile=\"http://example.org/schema\"" =~ def.jsonContentType.pattern
positive_match: "application/ld+json; charset=utf-8; foo=bar" =~ def.jsonContentType.pattern

positive_match: "application/cbor" =~ def.cborContentType.pattern
positive_match: "application/ce+cbor; foo=bar" =~ def.cborContentType.pattern

positive_match: "application/cddl" =~ def.cddlContentType.pattern
positive_match: "application/schema+cddl; charset=utf-8" =~ def.cddlContentType.pattern


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

negative_match: "B" =~ def.cardName.pattern
negative_match: "a_value" =~ def.cardName.pattern
negative_match: "a-namespace\\a-value" =~ def.cardName.pattern
negative_match: "a-namespace:sub-namespace:a-new-name" =~ def.cardName.pattern

negative_match: "a" =~ def.titleCaseName.pattern
negative_match: "A.Title" =~ def.titleCaseName.pattern
negative_match: "A title Case" =~ def.titleCaseName.pattern
negative_match: "A Title Case-name" =~ def.titleCaseName.pattern

negative_match: "application/cbor" =~ def.jsonContentType.pattern
negative_match: "application/cddl; charset=UTF-8" =~ def.jsonContentType.pattern

negative_match: "application/json" =~ def.cborContentType.pattern
negative_match: "application/ce+cddl; foo=bar" =~ def.cborContentType.pattern

negative_match: "application/cbor" =~ def.cddlContentType.pattern
negative_match: "application/schema+json; charset=utf-8" =~ def.cddlContentType.pattern
