package url

import (
	"net"
	"github.com/input-output-hk/catalyst-libs/specs/regex"
)

#absUrl: string
#absUrl: net.AbsURL

#absHttpsUrl: #absUrl
#absHttpsUrl: =~regex.def.httpsUrl.pattern

#relativeUrl: string
#relativeUrl: net.URL
