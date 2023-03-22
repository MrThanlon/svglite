cmake_minimum_required(VERSION 2.8)

project(fake-vglite)

set(CMAKE_BUILD_TYPE Debug)
set(CMAKE_C_COMPILER clang)
set(CMAKE_CXX_COMPILER clang++)

include_directories(../include)
add_executable(${PROJECT_NAME} main.c)
target_link_directories(${PROJECT_NAME} PRIVATE ../ ../target/debug)
target_link_libraries(${PROJECT_NAME} PRIVATE svglite)
