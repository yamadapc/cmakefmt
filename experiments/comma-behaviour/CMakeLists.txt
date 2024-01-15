SET(INPUT_VAR "bbbaaabbb")
string(REPLACE "b" "a", OUTPUT_VAR  "${INPUT_VAR}")
message(STATUS "INPUT: ${INPUT_VAR} OUTPUT: ${OUTPUT_VAR}")
