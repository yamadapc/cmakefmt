install(
  CODE
  "EXECUTE_PROCESS(\"\\$ENV{DESTDIR}\\${CMAKE_INSTALL_PREFIX}/samples/${sample_dir}\"\n                                   )"
)