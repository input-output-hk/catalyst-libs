// Master list of all document types.
package signed_docs

import (
	"list"
	"strings"
)

// Named Type UUIDs for easier definitions/references
// Note, not all documents are guaranteed to ever exist.
// IF the document is not explicitly defined, then these document
// types are placeholders subject to change.
_allDocTypes: {
	"Proposal Form Template":                 "0ce8ab38-9258-4fbc-a62e-7faa6e58318f" // rs
	"Proposal Presentation Template":         "cb99b9bd-681a-49d8-9836-89107c02e8ef" // new and speculative
	Proposal:                                 "7808d2ba-d511-40af-84e8-c0d1625fdfdc" // rs
	"Proposal Comment Form Template":         "0b8424d4-ebfd-46e3-9577-1775a69d290c" //rs
	"Proposal Comment Presentation Template": "eff42c84-f862-4573-bca1-5bc9584a241b" // new and speculative
	"Proposal Comment":                       "b679ded3-0e7c-41ba-89f8-da62a17898ea" // rs
	"Proposal Submission Action":             "5e60e623-ad02-4a1b-a1ac-406db978ee48" // rs
	"Proposal Moderation Action":             "a552451a-8e5b-409d-83a0-21eac26bbf8c" // new

	"Comment Moderation Action": "84a4b502-3b7e-47fd-84e4-6fee08794bd7" // new and speculative

	"Brand Parameters":                  "3e4808cc-c86e-467b-9702-d60baa9d1fca" // rs
	"Brand Parameters Form Template":    "fd3c1735-80b1-4eea-8d63-5f436d97ea31" // rs
	"Campaign Parameters":               "0110ea96-a555-47ce-8408-36efe6ed6f7c" // rs
	"Campaign Parameters Form Template": "7e8f5fa2-44ce-49c8-bfd5-02af42c179a3" // rs
	"Category Parameters":               "48c20109-362a-4d32-9bba-e0a9cf8b45be" // rs
	"Category Parameters Form Template": "65b1e8b0-51f1-46a5-9970-72cdf26884be" // rs
	"Contest Parameters":                "788ff4c6-d65a-451f-bb33-575fe056b411" // new
	"Contest Parameters Form Template":  "08a1e16d-354d-4f64-8812-4692924b113b" // new

	"Rep Profile":               "0f2c86a2-ffda-40b0-ad38-23709e1c10b3" //new
	"Rep Profile Form Template": "564cbea3-44d3-4303-b75a-d9fdda7e5a80" // new
	//"Rep Profile Moderation Action":    "0e20010b-eeaf-4938-a7ee-ceb3df9e8af6" // speculative
	"Rep Nomination":               "bf9abd97-5d1f-4429-8e80-740fea371a9c" // new
	"Rep Nomination Form Template": "431561a5-9c2b-4de1-8e0d-78eb4887e35d" // new
	//"Rep Nomination Moderation Action": "d27ecb44-bd4d-42bb-9273-5e5433cdfdb6" // speculative
	"Contest Delegation": "764f17fb-cc50-4979-b14a-b213dbac5994" // new
}

// Ensure that all Document Type IDs are Unique.
// See: all_docs.cue for a list of all known document types.
#allDocTypeIDs: list.UniqueItems

#allDocTypeIDs: [...#docType] & [
	for _, v in _allDocTypes {v},
]

_allDocNamesList: [...string] & [
	for k, _ in _allDocTypes {k},
]

// List of all the document names we have defined.
_allDocNames: or(_allDocNamesList)

// Individual Valid Document Name constraint.
#DocumentName: _allDocNames

// Document Cluster Definition
#DocumentCluster: {
	docs: [...#DocumentName]
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

// List of all Comment Docs (not templates or actions)
#commentDocNamesList: [...string] & [
	for k, _ in _allDocTypes
	if strings.Contains(k, "Comment") &&
		!strings.Contains(k, "Template") &&
		!strings.Contains(k, "Action") {k},
]

// List of all Parameters Docs (not templates or actions)
#parameterDocNamesList: [...string] & [
	for k, _ in _allDocTypes
	if strings.Contains(k, "Parameter") &&
		!strings.Contains(k, "Template") &&
		!strings.Contains(k, "Action") {k},
]

// List of all Template Docs (not actions)
#templateDocNamesList: [...string] & [
	for k, _ in _allDocTypes
	if strings.Contains(k, "Template") &&
		!strings.Contains(k, "Presentation") &&
		!strings.Contains(k, "Action") {k},
]
