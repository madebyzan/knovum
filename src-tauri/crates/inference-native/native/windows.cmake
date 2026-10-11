set(CMAKE_INSTALL_SYSTEM_RUNTIME_DESTINATION bin)
include(InstallRequiredSystemLibraries)
if(NOT CMAKE_INSTALL_SYSTEM_RUNTIME_LIBS)
  message(FATAL_ERROR "Licensed MSVC redistributable libraries are unavailable")
endif()
string(JSON sdk GET "${audio_record}" vulkanSdk)
string(JSON sdk_url GET "${sdk}" url)
string(JSON sdk_hash GET "${sdk}" sha256)
ExternalProject_Add(vulkan_sdk
  URL "${sdk_url}" URL_HASH "SHA256=${sdk_hash}" TLS_VERIFY ON DOWNLOAD_NO_EXTRACT TRUE
  CONFIGURE_COMMAND ""
  BUILD_COMMAND ${CMAKE_COMMAND} -E rm -rf "${CMAKE_BINARY_DIR}/vulkan-sdk"
    COMMAND <DOWNLOADED_FILE> --root "${CMAKE_BINARY_DIR}/vulkan-sdk"
      --accept-licenses --default-answer --confirm-command install copy_only=1
  INSTALL_COMMAND "")
list(APPEND audio_dependencies vulkan_sdk)
list(APPEND audio_flags -DENGINE_ENABLE_VULKAN=ON -DVulkan_INCLUDE_DIR=${CMAKE_BINARY_DIR}/vulkan-sdk/Include
  -DVulkan_LIBRARY=${CMAKE_BINARY_DIR}/vulkan-sdk/Lib/vulkan-1.lib
  -DVulkan_GLSLC_EXECUTABLE=${CMAKE_BINARY_DIR}/vulkan-sdk/Bin/glslc.exe)
string(JSON loader GET "${windows_record}" vulkanLoader)
knovum_download("${loader}" "${CMAKE_BINARY_DIR}/vulkan-loader.zip")
file(ARCHIVE_EXTRACT INPUT "${CMAKE_BINARY_DIR}/vulkan-loader.zip" DESTINATION "${CMAKE_BINARY_DIR}/vulkan-loader")
string(JSON entry GET "${loader}" entry)
string(JSON license GET "${loader}" licenseEntry)
install(FILES "${CMAKE_BINARY_DIR}/vulkan-loader/${entry}" DESTINATION bin)
install(FILES "${CMAKE_BINARY_DIR}/vulkan-loader/${license}" DESTINATION source-notices/windows-runtime RENAME VULKAN-LOADER-LICENSE.txt)
file(WRITE "${CMAKE_BINARY_DIR}/MSVC-RUNTIME.txt"
  "Microsoft Visual C++ v14 runtime: unmodified Microsoft-signed redistributable files from the licensed MSVC build toolchain, selected by CMake InstallRequiredSystemLibraries. Distribution requires release review.\nhttps://learn.microsoft.com/en-us/cpp/windows/redistributing-visual-cpp-files\nhttps://aka.ms/VCRedistLicense\n")
install(FILES "${CMAKE_BINARY_DIR}/MSVC-RUNTIME.txt" DESTINATION source-notices/windows-runtime)
install(FILES "${CMAKE_CURRENT_SOURCE_DIR}/windows-source.json" DESTINATION build-evidence)
