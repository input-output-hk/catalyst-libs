// List of all valid ICON choices.
// The UI is free to render Icons or not and render them in its own style.
// However, Icons not listed here are not supported.
@extern(embed)
package form_template

import (
	"list"
	"strings"
	"github.com/input-output-hk/catalyst-libs/specs/regex"
)

#iconName: string
#iconName: =~regex.def.iconName.pattern

_iconsFromAssets: _ @embed(glob="icons/*.svg",type=text)

_iconsSvg: {
	for filename, svg in _iconsFromAssets
	let no_extension = strings.TrimSuffix(filename, ".svg")
	let icon_name = strings.TrimPrefix(no_extension, "icons/") {
		"\(icon_name)": svg
	}
}

_allIcons: list.UniqueItems
_allIcons: [...#iconName]
_allIcons: list.Sort([
	for icon_name, svg in _iconsSvg {icon_name},
], list.Ascending)
#iconChoices: or(_allIcons)

allIcons:    _allIcons
allIconsSvg: _iconsSvg

// test
// good_icon: #iconChoices & "tag"
// bad_icon: #iconChoices & "tags"
