// Master list of all document types.
package signed_docs

import "list"

// Source of truth for ALL Document Types and their matching UUID's.
// Format is : <name> : <Document Type UUID>
_allDocs: {
	"Template": "7808d2ba-d511-40af-84e8-c0d1625fdfdc"
	"Proposal": "0ce8ab38-9258-4fbc-a62e-7faa6e58318f"
	"Comment":  "b679ded3-0e7c-41ba-89f8-da62a17898ea"
	"Action":   "5e60e623-ad02-4a1b-a1ac-406db978ee48"
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
