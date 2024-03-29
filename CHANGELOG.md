# 0.1.11 (15-01-2024)

* Fix parsing files that start with leading spaces
* Fix parsing `TEST` unary operator in conditions
* Fix parsing lower-case operators (`and`, `or` etc)
* Parse bracket strings (multiline strings)
* Ignore trailing commas after string literals `"string",`
  - This technically changes the meaning of the document
  - However, the trailing comma is invalid syntax

# 0.1.10 (02-01-2024)

* Fix parenthesis bugs
* Fix bracket comment edge-case
* Fix comment newline handling bugs
* Fix parsing comments within conditions

# 0.1.9 (31-12-2023)

* Fixes to parsing parenthesis in commands
* Fixes to parsing strings and escape sequences
* Adds support for CMake multi-line bracket comments

# 0.1.8 (14-12-2023)

* Fixes to parsing errors and improvements to error reporting

# 0.1.7 (13-12-2023)

* Add in-place flag

# 0.1.6 (13-12-2023)

* Improve quality of formatting boolean expressions

# 0.1.5 (11-12-2023)

* Fix bugs with nested parenthesis and if statement arg printing

# 0.1.4 (10-12-2023)

* Improve output quality

# 0.1.3 (10-12-2023)

* Fix foreach handling

# 0.1.2 (10-12-2023)

* Fix broken variable handling

# 0.1.1 (10-12-2023)

* Fix if statements and some parenthesis handling

# 0.1.0 (10-12-2023)

* Initial release
