cmake_minimum_required(VERSION 3.10)

project(annabella VERSION 0.1.0 LANGUAGES C)

add_compile_definitions(ANNABELLA_VERSION="${annabella_VERSION}")

set(CMAKE_EXECUTABLE_ENABLE_EXPORTS TRUE)
set(CMAKE_EXPORT_COMPILE_COMMANDS TRUE)
set(CMAKE_C_VISIBILITY_PRESET "hidden")

file(GLOB_RECURSE SRCFILES "transpiler/*.c")

add_executable(annabella ${SRCFILES})

file(GLOB_RECURSE SRCFILES "runtime/*.c")

add_library(annabella-rt SHARED ${SRCFILES})
set_target_properties(annabella-rt PROPERTIES PUBLIC_HEADER "runtime/annabella-rt.h")

install(TARGETS annabella)
install(TARGETS annabella-rt
  PUBLIC_HEADER DESTINATION "${CMAKE_INSTALL_INCLUDEDIR}"
)
