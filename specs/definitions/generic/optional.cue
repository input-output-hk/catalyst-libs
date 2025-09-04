package optional

#field_without_default: 
					"yes" |
					"optional" |
					"excluded"

// Is a field Required, Optional or Excluded/Unused
#field: #field_without_default | *"excluded"
