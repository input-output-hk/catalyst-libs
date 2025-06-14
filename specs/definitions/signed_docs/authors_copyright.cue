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
			version:  "0.01"
			modified: "2025-04-04"
			changes: """
				* First Published Version
				"""
		},
		{
			version:  "0.02"
			modified: "2025-04-09"
			changes: """
				* Add version control changelogs to the specification.
				"""
		},
		{
			version:  "0.03"
			modified: "2025-05-05"
			changes: """
				* Use generalized parameters.
				"""
		},
		{
			version:  "0.04"
			modified: "2025-05-30"
			changes: """
				* Improve and make document serialization more repeatable, and stricter.
				* TODO: Define Systems parameters
				* TODO: Define DReps documents.
				* TODO: Define Proposer Profiles.
				* TODO: Define Role 0 Profile.
				"""
		},
	]
}
