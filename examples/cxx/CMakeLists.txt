cmake_minimum_required(VERSION 3.14)
project(millennium-cxx-demo)

include(.millennium/millennium.cmake)

set(CMAKE_EXPORT_COMPILE_COMMANDS ON)

add_executable(millennium-cxx-demo src/main.cc)
target_include_directories(millennium-cxx-demo PUBLIC millennium)
target_link_libraries(millennium-cxx-demo millennium)
target_compile_definitions(millennium-cxx-demo PRIVATE UNICODE _UNICODE)
