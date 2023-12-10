for i in samples/*.input.cmake; do
  name=`basename -s ".input.cmake" $i`
  cargo run $i > samples/$name.output.cmake
done
