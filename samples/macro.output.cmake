macro(bar)

  
  foreach(arg IN LISTS ARGN)
  
    
    baz()
    
  
  endforeach()
  

endmacro()
