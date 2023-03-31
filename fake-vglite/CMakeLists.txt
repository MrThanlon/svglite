cmake_minimum_required(VERSION 3.0)

project(fake-vglite)

set(CMAKE_BUILD_TYPE Debug)

include_directories(../include)
add_executable(${PROJECT_NAME} main.c)
target_link_directories(${PROJECT_NAME} PRIVATE ../target/debug)
target_link_libraries(${PROJECT_NAME} PRIVATE svglite)
