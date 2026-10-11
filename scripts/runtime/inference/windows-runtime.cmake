cmake_minimum_required(VERSION 4.3)
project(KnovumRuntimeSupply LANGUAGES CXX)
include(InstallRequiredSystemLibraries)
if(NOT CMAKE_INSTALL_SYSTEM_RUNTIME_LIBS)
  message(FATAL_ERROR "MSVC redistributable libraries are unavailable")
endif()
file(WRITE "${CMAKE_BINARY_DIR}/compiler.json"
  "{\"id\":\"${CMAKE_CXX_COMPILER_ID}\",\"version\":\"${CMAKE_CXX_COMPILER_VERSION}\",\"toolset\":\"${MSVC_TOOLSET_VERSION}\"}")
