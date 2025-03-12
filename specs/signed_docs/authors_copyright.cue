// Signed Document Definitions
// 
// COSE Headers and Constraints
package signed_docs

// List of authors, name: email
#authorList: {
	[string]: string
}

// General Authors List
authors: #authorList & {
	"Steven Johnson":   "steven.johnson@iohk.io"
	"Alex Pozhylenkov": "alex.pozhylenkov@iohk.io"
}

#copyrightNotice: {
	created:   string // YYYY-MM-DD
	license:   "CC-BY-4.0"
	copyright: "IOG Singapore, All Rights Reserved"
}

copyright: #copyrightNotice & {
	created: "2024-12-27"
}
