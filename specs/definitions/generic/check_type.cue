/// Check Type and Value as a string or an array.
package check_type

this: "string"
that: [["object"], null]

is_this: this.Kind()
is_that: that.Kind()
