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
		{
			version:  "0.1.1"
			modified: "2025-08-19"
			changes: """
				* Define an Optional Section in a Form Template, to enable partial form submission while in Draft.
				"""
		},
		{
			version:  "0.1.2"
			modified: "2025-09-08"
			changes: """
				* Updated `payload` field, it become required.
				* Added new `draft` field for Signed Document with the default value `false`.
				* Made `payload.nil` non optional with the default value `false`.
				* If `payload.nil` is `true` automatically set `"content type"` and `"content-encoding"` fields to `"excluded"`.
				"""
		},
		{
			version:  "0.1.3"
			modified: "2025-09-09"
			changes: """
				* Fixed an invalid 'Presentation Template' JSON schema. 
				"""
		},
		{
			version:  "0.1.4"
			modified: "2025-10-17"
			changes: """
				* Modified `collaborators` cddl definition, it must have at least one element in array.
				"""
		},
		{
			version:  "0.1.5"
			modified: "2025-10-24"
			changes: """
				* Updated 'Proposal Submission Action' document, set `ref` metadata field `multiply` property to `false`.
				* Changed spec `signers.update` property structure.
				"""
		},
		{
			version:  "0.2.0"
			modified: "2025-11-10"
			changes: """
				* Added a new 'Contest Ballot' and 'Contest Ballot Checkpoint' document types.
				* Improved the specification for 'Contest Delegation' document type.
				* 'content encoding' metadata field become non optional for all document types where it was an optional field.
				* Added new 'payload.schema' type - CDDL schema, defined as string.
				"""
		},
		{
			version:  "0.2.1"
			modified: "2025-12-02"
			changes: """
				* Added missing `ref` metadata field definition.
				* Improved `payload` cddl definition, replaced `document_ref` to the `uint` as a map keys to the `choices`.
				"""
		},
		{
			version:  "0.2.2"
			modified: "2025-12-15"
			changes: """
				* Added missing `signers: update: type: "ref"` definition for `Rep Nomination` document type.
				"""
		},
		{
			version:  "0.2.3"
			modified: "2026-01-09"
			changes: """
				* Internal dependency updates.
				"""
		},
		{
			version:  "0.2.4"
			modified: "2026-01-13"
			changes: """
				* Internal dependency updates.
				"""
		},
		{
			version:  "0.2.5"
			modified: "2026-01-15"
			changes: """
				* `catalyst-signed-doc-spec` payload `Schema::Json` type.
				"""
		},
	]
}
