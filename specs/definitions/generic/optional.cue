package optional

#field:
	"yes" |
	"optional" |
	"excluded"

// Is a field Required, Optional or Excluded/Unused
#field_default_yes:      #field | *"yes"
#field_default_optional: #field | *"optional"
#field_default_excluded: #field | *"excluded"
