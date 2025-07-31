// Regex Definitions
package regex

import (
	"regexp"
)

#regex: string
#regex: regexp.Valid

#regexName: string
#regexName: =~_camelCaseName

#def: [#regexName]: {
	pattern:     #regex
	description: string
}
