cmake_minimum_required(VERSION 4.3)
project(KnovumInferenceLibraries LANGUAGES C CXX)
include(ExternalProject)
include(sources.cmake)
string(JSON required_cmake GET "${audio_record}" cmakeVersion)
if(NOT CMAKE_VERSION VERSION_EQUAL required_cmake)
  message(FATAL_ERROR "The qualified compiled toolchain requires CMake ${required_cmake}")
endif()
include(onnx.cmake)
set(common -DCMAKE_BUILD_TYPE=Release -DCMAKE_INSTALL_PREFIX=${CMAKE_INSTALL_PREFIX})
if(APPLE)
  list(APPEND common -DCMAKE_OSX_DEPLOYMENT_TARGET=14.0 -DCMAKE_OSX_ARCHITECTURES=${CMAKE_OSX_ARCHITECTURES} -DCMAKE_INSTALL_RPATH=@loader_path)
endif()
# Suppliers nest their own ExternalProject trees and MSVC resolves intermediate files through the
# 260-character path limit, so every supplier uses the shortest source/build layout CMake allows.
function(knovum_layout name)
  set(${name}_layout PREFIX "${CMAKE_BINARY_DIR}/${name}" SOURCE_DIR "${CMAKE_BINARY_DIR}/${name}/s"
    BINARY_DIR "${CMAKE_BINARY_DIR}/${name}/b" PARENT_SCOPE)
endfunction()
foreach(name audio sherpa whisper)
  knovum_archive(${name} "${${name}_record}")
  knovum_layout(${name})
endforeach()
set(audio_flags -DENGINE_ENABLE_METAL=${APPLE} -DENGINE_ENABLE_CUDA=OFF -DENGINE_ENABLE_VULKAN=OFF
  -DGGML_METAL_MACOSX_VERSION_MIN=14.0)
set(audio_dependencies)
foreach(name AVX AVX2 FMA F16C BMI2)
  list(APPEND audio_flags -DGGML_${name}=OFF)
endforeach()
if(WIN32)
  include(windows.cmake)
endif()
ExternalProject_Add(audio ${audio_layout} BUILD_ALWAYS TRUE DEPENDS ${audio_dependencies}
  URL "${audio_url}" URL_HASH "${audio_hash}" TLS_VERIFY ON
  DOWNLOAD_EXTRACT_TIMESTAMP FALSE
  CONFIGURE_COMMAND ${CMAKE_COMMAND} -S ${CMAKE_CURRENT_SOURCE_DIR}/audio -B <BINARY_DIR>
    -DKNOVUM_AUDIOCPP_SOURCE=<SOURCE_DIR> -DKNOVUM_VOICE_BRIDGE=${CMAKE_CURRENT_SOURCE_DIR}/audio/voice_bridge.cpp ${common} ${audio_flags}
  BUILD_COMMAND ${CMAKE_COMMAND} --build <BINARY_DIR> --config Release --parallel 4
  INSTALL_COMMAND ${CMAKE_COMMAND} --install <BINARY_DIR> --config Release --component knovum-voice
    COMMAND ${CMAKE_COMMAND} -DSOURCE=<SOURCE_DIR> -DPREFIX=${CMAKE_INSTALL_PREFIX} -P ${CMAKE_CURRENT_SOURCE_DIR}/audio/notices.cmake)
ExternalProject_Add(sherpa ${sherpa_layout} BUILD_ALWAYS TRUE DEPENDS audio
  URL "${sherpa_url}" URL_HASH "${sherpa_hash}" TLS_VERIFY ON DOWNLOAD_EXTRACT_TIMESTAMP FALSE
  CONFIGURE_COMMAND ${CMAKE_COMMAND} -E env
    SHERPA_ONNXRUNTIME_INCLUDE_DIR=${CMAKE_BINARY_DIR}/ort-headers SHERPA_ONNXRUNTIME_LIB_DIR=${CMAKE_BINARY_DIR}/ort-lib
    ${CMAKE_COMMAND} -S <SOURCE_DIR> -B <BINARY_DIR> ${common}
    -DBUILD_SHARED_LIBS=ON -DSHERPA_ONNX_ENABLE_PYTHON=OFF -DSHERPA_ONNX_ENABLE_TTS=OFF
    -DSHERPA_ONNX_ENABLE_PORTAUDIO=OFF -DSHERPA_ONNX_ENABLE_WEBSOCKET=OFF -DSHERPA_ONNX_ENABLE_BINARY=OFF
    -DSHERPA_ONNX_ENABLE_EXAMPLES=OFF -DSHERPA_ONNX_ENABLE_C_API=ON -DSHERPA_ONNX_ENABLE_SPEAKER_DIARIZATION=ON
  BUILD_COMMAND ${CMAKE_COMMAND} --build <BINARY_DIR> --config Release --parallel 4
  INSTALL_COMMAND ${CMAKE_COMMAND} -DSOURCE=<SOURCE_DIR> -DBUILD=<BINARY_DIR> -DPREFIX=${CMAKE_INSTALL_PREFIX} -DKIND=sherpa -P ${CMAKE_CURRENT_SOURCE_DIR}/install.cmake)
set(whisper_flags -DBUILD_SHARED_LIBS=ON -DGGML_BACKEND_DL=ON -DGGML_CPU_ALL_VARIANTS=ON
  -DGGML_NATIVE=OFF -DGGML_OPENMP=OFF -DWHISPER_BUILD_TESTS=OFF -DWHISPER_BUILD_EXAMPLES=OFF -DWHISPER_BUILD_SERVER=OFF)
if(KNOVUM_TARGET STREQUAL "macos-arm64")
  list(APPEND whisper_flags -DGGML_METAL=ON)
else()
  list(APPEND whisper_flags -DGGML_METAL=OFF)
endif()
foreach(name AVX AVX2 FMA F16C BMI2)
  list(APPEND whisper_flags -DGGML_${name}=OFF)
endforeach()
ExternalProject_Add(whisper ${whisper_layout} BUILD_ALWAYS TRUE DEPENDS sherpa
  URL "${whisper_url}" URL_HASH "${whisper_hash}" TLS_VERIFY ON DOWNLOAD_EXTRACT_TIMESTAMP FALSE
  CMAKE_ARGS ${common} ${whisper_flags}
  BUILD_COMMAND ${CMAKE_COMMAND} --build <BINARY_DIR> --config Release --parallel 4
  INSTALL_COMMAND ${CMAKE_COMMAND} -DSOURCE=<SOURCE_DIR> -DBUILD=<BINARY_DIR> -DPREFIX=${CMAKE_INSTALL_PREFIX} -DKIND=whisper -P ${CMAKE_CURRENT_SOURCE_DIR}/install.cmake)
install(FILES ${CMAKE_CURRENT_SOURCE_DIR}/onnx-source.json ${CMAKE_CURRENT_SOURCE_DIR}/audio-source.json
  ${CMAKE_CURRENT_SOURCE_DIR}/sherpa-source.json ${CMAKE_CURRENT_SOURCE_DIR}/whisper-source.json DESTINATION build-evidence)
configure_file(build-evidence.json.in build-evidence.json @ONLY)
install(FILES "${CMAKE_BINARY_DIR}/build-evidence.json" DESTINATION build-evidence)
install(DIRECTORY "${CMAKE_CURRENT_SOURCE_DIR}/" DESTINATION build-evidence/cmake-build-inputs
  FILES_MATCHING PATTERN "*.cmake" PATTERN "CMakeLists.txt" PATTERN "*.cpp" PATTERN "*.in")

if(WIN32)
  if(NOT CMAKE_DUMPBIN)
    get_filename_component(_cl_dir "${CMAKE_C_COMPILER}" DIRECTORY)
    find_program(CMAKE_DUMPBIN NAMES dumpbin HINTS "${_cl_dir}")
  endif()
  configure_file(windows-qualification.cmake.in windows-qualification.cmake @ONLY)
  install(SCRIPT "${CMAKE_BINARY_DIR}/windows-qualification.cmake")
  install(FILES "${CMAKE_BINARY_DIR}/windows-qualification.cmake" DESTINATION build-evidence)
endif()
