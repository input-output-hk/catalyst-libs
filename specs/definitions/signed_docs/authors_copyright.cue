// Signed Document Definitions
// 
// COSE Headers and Constraints
package signed_docs

import (
	"github.com/input-output-hk/catalyst-libs/specs/generic:date"
)

// List of authors, name: email
#authorList: {
	[string]: string
}

// 
#authorMinConstraint: {}

// General Authors List
authors: #authorList & {
	"Steven Johnson":   "steven.johnson@iohk.io"
	"Alex Pozhylenkov": "alex.pozhylenkov@iohk.io"
	"Nathan Bogale":    "nathan.bogale@iohk.io"
	"Neil McAuliffe":   "neil.mcauliffe@iohk.io"
}

// Individual Changelog Entry
#changelogEntry: {
	version:  string
	modified: date.#yyyymmdd // YYYY-MM-DD
	changes:  string
}

// Copyright Notice
#copyrightNotice: {
	created:   date.#yyyymmdd // YYYY-MM-DD
	license:   "CC-BY-4.0"
	copyright: "IOG Singapore, All Rights Reserved"
	versions: [...#changelogEntry]
}

copyright: #copyrightNotice & {
	created: "2024-12-27"
	versions: [
		{
			version:  "0.0.1"
			modified: "2025-04-04"
			changes: """
				* First Published Version
				"""
		},
		{
			version:  "0.0.2"
			modified: "2025-04-09"
			changes: """
				* Add version control changelogs to the specification.
				"""
		},
		{
			version:  "0.0.3"
			modified: "2025-05-05"
			changes: """
				* Use generalized parameters.
				"""
		},
		{
			version:  "0.0.4"
			modified: "2025-06-25"
			changes: """
				* Improve and make document serialization more repeatable, and stricter.
				* Defined Systems parameters documents
				* Defined DReps documents.
				"""
		},
		{
			version:  "0.1.0"
			modified: "2025-07-30"
			changes: """
				* Fixed typographical and layout issues.
				* Added specifications for Form Templates and Elements.
				* Reverted Document Types to a Single UUID from an Array of UUIDs
				* Changed versions to be semantic (0.04 became 0.0.4)
				* Corrected Parameter References for Brand/Campaign/Category/Contest Templates
				* Replaced poorly formatting markdown tables with HTML equivalents.
				* Added draft placeholders for Moderation Documents (subject to change)
				* Clarified How Representatives may delegate.
				* Clarified what happens when a new version of a Nomination Document is published.
				* Clarified how delegations can be revoked.
				* Clarified the payload for delegations.
				* Generalized Presentation Templates, and defined sample (subject to change) cards.
				* Removed specialized presentation templates, as a universal presentation template is all thats required.
				* Converted draft-7 Json Schemas to 2020-12
				* Add standard ICON definitions for Forms.
				"""
		},
	]
}
