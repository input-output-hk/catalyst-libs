// Master list of all document types.
package signed_docs

import "list"

// Named Type UUIDs for easier definitions/references
_allDocTypes: {
	"Template":         "0ce8ab38-9258-4fbc-a62e-7faa6e58318f"
	"Proposal":         "7808d2ba-d511-40af-84e8-c0d1625fdfdc"
	"Comment":          "b679ded3-0e7c-41ba-89f8-da62a17898ea"
	"Action":           "5e60e623-ad02-4a1b-a1ac-406db978ee48"
	"SubmissionAction": "78927329-cfd9-4ea1-9c71-0e019b126a65"
	"ModerationAction": "a5d232b8-5e03-4117-9afd-be32b878fcdd"
	"Category":         "818938c3-3139-4daa-afe6-974c78488e95"
}

// Ensure that all Document Type IDs are Unique.
// See: all_docs.cue for a list of all known document types.
_allDocTypeIDs: list.UniqueItems
_allDocTypeIDs: [...#uuidv4] & [
	for _, v in _allDocTypes {v},
]

// Source of truth for ALL Document Types and their matching UUID's.
// Format is : <name> : <Document Type UUID>
_allDocs: {
	"Proposal Meta Template": [
		_allDocTypes["Template"], // Template
		_allDocTypes["Template"], // For Templates
		_allDocTypes["Proposal"], // On Proposals
	]
	"Proposal Template": [
		_allDocTypes["Template"], // Template
		_allDocTypes["Proposal"], // For Proposals
	]
	"Proposal": [
		_allDocTypes["Proposal"],
	]
	"Proposal Comment Template": [
		_allDocTypes["Template"], // Template
		_allDocTypes["Comment"],  // For Comments
		_allDocTypes["Proposal"], // On Proposals
	]
	"Proposal Comment": [
		_allDocTypes["Comment"],  // Comment
		_allDocTypes["Proposal"], // For Proposals
	]
	"Proposal Submission Action":
	[
		_allDocTypes["Action"],           // Action
		_allDocTypes["Proposal"],         // For Proposal
		_allDocTypes["SubmissionAction"], // On Submission
	]
	"Proposal Moderation Action":
	[
		_allDocTypes["Action"],           // Action
		_allDocTypes["Proposal"],         // For Proposal
		_allDocTypes["ModerationAction"], // On Moderation
	]
	"Comment Action Document": [
		_allDocTypes["Action"],           // Action
		_allDocTypes["Comment"],          // For Comment
		_allDocTypes["ModerationAction"], // On Moderation
	]
	"Category Parameters": [
		_allDocTypes["Category"],
	]

}

// Ensure that all Document IDs are Unique.
// See: all_docs.cue for a list of all known document types.
_allTypes: list.UniqueItems
_allTypes: [...#docType] & [
	for _, v in _allDocs {v},
]

_allDocNamesList: [...string] & [
	for k, _ in _allDocs {k},
]

// List of all the document names we have defined.
_allDocNames: or(_allDocNamesList)

// Individual Valid Document Name constraint.
#DocumentName: _allDocNames
