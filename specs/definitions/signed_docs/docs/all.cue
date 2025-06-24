// Master list of all document types.
package signed_docs

import (
	"list"
)

// Named Type UUIDs for easier definitions/references
_allDocTypes: {
	FormTemplate:         "0ce8ab38-9258-4fbc-a62e-7faa6e58318f"
	PresentationTemplate: "cb99b9bd-681a-49d8-9836-89107c02e8ef"
	Proposal:             "7808d2ba-d511-40af-84e8-c0d1625fdfdc"
	Comment:              "b679ded3-0e7c-41ba-89f8-da62a17898ea"
	Action:               "5e60e623-ad02-4a1b-a1ac-406db978ee48"
	SubmissionAction:     "78927329-cfd9-4ea1-9c71-0e019b126a65"
	ModerationAction:     "a5d232b8-5e03-4117-9afd-be32b878fcdd"
	Brand:                "ebcabeeb-5bc5-4f95-91e8-cab8ca724172"
	Campaign:             "5ef32d5d-f240-462c-a7a4-ba4af221fa23"
	Category:             "818938c3-3139-4daa-afe6-974c78488e95"
	Contest:              "788ff4c6-d65a-451f-bb33-575fe056b411"
	Parameters:           "60185874-7e13-407c-a06c-238ffe637ae6"
	RegisteredUser:       "ff4b7724-3db5-44cd-a433-78ba6d29505e"
	RegisteredRep:        "94579df1-a6dc-433b-a8e8-910c5dc2f0e3"
	RegisteredProposer:   "7311c63b-95c6-402e-a258-f9bf622093eb"
	Profile:              "0f2c86a2-ffda-40b0-ad38-23709e1c10b3"
	Nomination:           "bf9abd97-5d1f-4429-8e80-740fea371a9c"
}

// Source of truth for ALL Document Types and their matching UUID's.
// Format is : <name> : <Document Type UUID>
_allDocs: {
	"Proposal Form Template": [
		_allDocTypes["FormTemplate"], // Form Template
		_allDocTypes["Proposal"],     // For Proposals
	]
	"Proposal Presentation Template": [
		_allDocTypes["PresentationTemplate"], // Presentation Template
		_allDocTypes["Proposal"],             // For Proposals
	]
	Proposal: [
		_allDocTypes["Proposal"],
	]
	"Proposal Comment Form Template": [
		_allDocTypes["FormTemplate"], // Form Template
		_allDocTypes["Comment"],      // For Comments
		_allDocTypes["Proposal"],     // On Proposals
	]
	"Proposal Comment Presentation Template": [
		_allDocTypes["PresentationTemplate"], // PresentationTemplate
		_allDocTypes["Comment"],              // For Comments
		_allDocTypes["Proposal"],             // On Proposals
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
		_allDocTypes["Parameters"], // Parameters
		_allDocTypes["Brand"],      // Of a Brand
	]
	"Brand Parameters Form Template": [
		_allDocTypes["FormTemplate"], // Form Template
		_allDocTypes["Parameters"],   // For Parameters
		_allDocTypes["Brand"],        // Of a Brand
	]
	"Campaign Parameters": [
		_allDocTypes["Parameters"], // Parameters
		_allDocTypes["Campaign"],   // Of a Campaign
	]
	"Campaign Parameters Form Template": [
		_allDocTypes["FormTemplate"], // Form Template
		_allDocTypes["Parameters"],   // For Parameters
		_allDocTypes["Campaign"],     // Of a Campaign
	]
	"Category Parameters": [
		_allDocTypes["Parameters"], // Parameters
		_allDocTypes["Category"],   // Of a Category
	]
	"Category Parameters Form Template": [
		_allDocTypes["FormTemplate"], // Form Template
		_allDocTypes["Parameters"],   // For Parameters
		_allDocTypes["Category"],     // Of a Category
	]
	"Contest Parameters": [
		_allDocTypes["Parameters"], // Parameters
		_allDocTypes["Contest"],    // Of a Contest
	]
	"Contest Parameters Form Template": [
		_allDocTypes["FormTemplate"], // Form Template
		_allDocTypes["Parameters"],   // For Parameters
		_allDocTypes["Contest"],      // Of a Contest
	]
	"Rep Profile": [
		_allDocTypes["Profile"],       // Profile
		_allDocTypes["RegisteredRep"], // Of a Registered Rep
	]
	"Rep Profile Form Template": [
		_allDocTypes["FormTemplate"],  // Form Template
		_allDocTypes["Profile"],       // For a Profile
		_allDocTypes["RegisteredRep"], // Of a Registered Rep
	]
	"Rep Nomination": [
		_allDocTypes["Nomination"],    // Nomination
		_allDocTypes["RegisteredRep"], // Of a Registered Rep
	]
	"Rep Nomination Form Template": [
		_allDocTypes["FormTemplate"],  // Form Template
		_allDocTypes["Nomination"],    // For a Nomination
		_allDocTypes["RegisteredRep"], // Of a Registered Rep
	]
}

// Document Cluster Definition
#DocumentCluster: {
	docs: [..._allDocNames]
}

#DocClusters: [string]: #DocumentCluster

doc_clusters: #DocClusters & {
	// System parameters define the system, excludes Contests.
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
