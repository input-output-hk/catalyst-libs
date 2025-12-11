// Master list of all document types.
package signed_doc_types

import (
	"list"
	"strings"
	"github.com/input-output-hk/catalyst-libs/specs/generic:uuid"
	"github.com/input-output-hk/catalyst-libs/specs/regex"
)

// Document Type must be a valid UUIDv4
#docType: uuid.#v4

// Document Name Constraint
#docName: string
#docName: =~regex.def.titleCaseName.pattern

// Ensure that all Document Type IDs are Unique.
// See: all_docs.cue for a list of all known document types.
allDocTypeIDs: list.UniqueItems
allDocTypeIDs: [...#docType] & [
	for _, v in allDocTypes {v},
]

// All the document Names
allDocNames: [...#docName] & [
	for k, _ in allDocTypes {k},
]

// Any one defined Document Name.
#allDocNames: or(allDocNames)

// Document Cluster Definition
#DocumentCluster: {
	docs: [...#docName]
}

#DocClusters: [string]: #DocumentCluster

doc_clusters: #DocClusters & {
	// System parameters define the system hierarchy.
	"System Parameters": {
		docs: [
			"Brand Parameters",
			"Campaign Parameters",
			"Category Parameters",
			"Contest Parameters",
		]
	}
}

// A Doc can only be in 1 cluster.
// Only used to validate that constraint.
allDocClusterDocs: list.UniqueItems
allDocClusterDocs: [...string] & [
	for cluster in doc_clusters
	for doc in cluster.docs {doc},
]

// List of all Comment Docs (not templates or actions)
commentDocNamesList: [...string] & [
	for k, _ in allDocTypes
	if strings.Contains(k, "Comment") &&
		!strings.Contains(k, "Template") &&
		!strings.Contains(k, "Action") {k},
]

// List of all Parameters Docs (not templates or actions)
parameterDocNamesList: [...string] & [
	for k, _ in allDocTypes
	if strings.Contains(k, "Parameter") &&
		!strings.Contains(k, "Template") &&
		!strings.Contains(k, "Action") {k},
]

// List of all Template Docs (not actions)
templateDocNamesList: [...string] & [
	for k, _ in allDocTypes
	if strings.Contains(k, "Template") &&
		!strings.Contains(k, "Presentation") &&
		!strings.Contains(k, "Action") {k},
]
