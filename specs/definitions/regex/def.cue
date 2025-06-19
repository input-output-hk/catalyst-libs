// Regex Definitions
package regex

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

// Negative match (where possible to test)
negative_match: false

negative_match: "multi\nline" =~ def.singleLine.pattern

// No negative multiline regex cases. Regex too simple.

negative_match: "" =~ def.httpsUrl.pattern
negative_match: "not a url" =~ def.httpsUrl.pattern
negative_match: "http://www.iana.org/assignments" =~ def.httpsUrl.pattern
