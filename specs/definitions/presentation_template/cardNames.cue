// Choices that can be made for a string field.
package presentation_template

import (
	"list"
	"github.com/input-output-hk/catalyst-libs/specs/regex"
	"github.com/input-output-hk/catalyst-libs/specs/signed_doc_types"
)

#cardName: string
#cardName: =~regex.def.cardName.pattern

#available_docs: list.MinItems(1)
#available_docs: [...signed_doc_types.#allDocNames]

#cardDefinition: {
	name:           string
	description:    string
	available_docs: #available_docs
}

allCards: [#cardName]: #cardDefinition

// Valid values for a cards Name
_allCardNames: list.UniqueItems
_allCardNames: list.Sort([
	for k, _ in allCards {k},
], list.Ascending)

#cardNameChoices: or(_allCardNames)
