// Date and Time Definitions
package date

// A Date in the format YYYY=MM-DD UUIDv4 formatted string regex
#yyyymmdd: =~"^(20((2[4-9])|([3-9][0-9])))-((((0[13578])|(1[02]))-(([0-2][0-9])|(3[01])))|(((0[469])|(11))-(([0-2][0-9])|(30)))|(02-[0-2][0-9]))$"

test: #yyyymmdd & "1234-12-12"
