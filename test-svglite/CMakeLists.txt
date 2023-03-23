cmake_minimum_required(VERSION 2.8)

project(svglite2bin)

set(CMAKE_C_COMPILER riscv64-unknown-linux-gnu-gcc)
set(CMAKE_CXX_COMPILER riscv64-unknown-linux-gnu-g++)

include_directories(../include)
add_executable(${PROJECT_NAME} main.c)
target_link_directories(${PROJECT_NAME} PRIVATE ../ ../target/riscv64gc-unknown-linux-gnu/debug)
target_link_libraries(${PROJECT_NAME} PRIVATE vg_lite svglite)
