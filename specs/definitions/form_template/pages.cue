// Template Json Schema Definitions Dictionary
// 
// All known and supported Json Schema definitions,
// and their parameters and documentation so that
// a dictionary document and the definitions themselves
// can be generated.
@extern(embed)

package form_template

import "strings"

#page: [string]: string

// Generic Form Template
pages: #page

// This has the pages with the "pages/" extension
_pages: _ @embed(glob="pages/*.*", type=text)

// This strips that extension out.
pages: {
	for page, content in _pages
	let name = strings.Replace(page, "pages/", "", 1) {
		"\(name)": content
	}
}
