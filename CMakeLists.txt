cmake_minimum_required(VERSION 3.10)

project(annabella VERSION 0.1.0 LANGUAGES C)

add_compile_definitions(ANNABELLA_VERSION="${annabella_VERSION}")

set(CMAKE_EXECUTABLE_ENABLE_EXPORTS TRUE)
set(CMAKE_EXPORT_COMPILE_COMMANDS TRUE)

file(GLOB_RECURSE SRCFILES "src/*.c")

add_executable(annabella ${SRCFILES})
