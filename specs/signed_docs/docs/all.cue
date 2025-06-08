// Master list of all document types.
package signed_docs

import (
	"list"
)

// Named Type UUIDs for easier definitions/references
_allDocTypes: {
	Template:         "0ce8ab38-9258-4fbc-a62e-7faa6e58318f"
	Proposal:         "7808d2ba-d511-40af-84e8-c0d1625fdfdc"
	Comment:          "b679ded3-0e7c-41ba-89f8-da62a17898ea"
	Action:           "5e60e623-ad02-4a1b-a1ac-406db978ee48"
	SubmissionAction: "78927329-cfd9-4ea1-9c71-0e019b126a65"
	ModerationAction: "a5d232b8-5e03-4117-9afd-be32b878fcdd"
	Brand:            "ebcabeeb-5bc5-4f95-91e8-cab8ca724172"
	Campaign:         "5ef32d5d-f240-462c-a7a4-ba4af221fa23"
	Category:         "818938c3-3139-4daa-afe6-974c78488e95"
	Decision:         "788ff4c6-d65a-451f-bb33-575fe056b411"
}

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
	Proposal: [
		_allDocTypes["Proposal"],
	]
	"Proposal Comment Meta Template": [
		_allDocTypes["Template"], // Template
		_allDocTypes["Template"], // For Templates
		_allDocTypes["Comment"],  // On Comment
		_allDocTypes["Proposal"], // On Proposals
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
	"Comment Moderation Action": [
		_allDocTypes["Action"],           // Action
		_allDocTypes["Comment"],          // For Comment
		_allDocTypes["ModerationAction"], // On Moderation
	]
	"Brand Parameters": [
		_allDocTypes["Brand"],
	]
	"Campaign Parameters": [
		_allDocTypes["Campaign"],
	]
	"Category Parameters": [
		_allDocTypes["Category"],
	]
	"Decision Parameters": [
		_allDocTypes["Decision"],
	]

}

// Document Cluster Definition
#DocumentCluster: {
	docs: [..._allDocNames]
}

#DocClusters: [string]: #DocumentCluster

doc_clusters: #DocClusters & {
	// System parameters define the system, excludes Decisions.
	"System Parameters": {
		docs: [
			"Brand Parameters",
			"Campaign Parameters",
			"Category Parameters",
		]
	}
}

// A Doc can only be in 1 cluster.
#allDocClusterDocs: list.UniqueItems
#allDocClusterDocs: [...string] & [
	for cluster in doc_clusters
	for doc in cluster.docs {doc},
]
