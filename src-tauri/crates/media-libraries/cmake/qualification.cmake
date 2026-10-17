# Verify the frozen supplier rather than accepting whichever FFmpeg is on PATH.
if(APPLE)
  execute_process(COMMAND otool -L "${executable}" OUTPUT_VARIABLE imports COMMAND_ERROR_IS_FATAL ANY)
  string(REGEX MATCHALL "\n\t[^\n]+" imports "${imports}")
  if(NOT imports)
    message(FATAL_ERROR "Media executable has no system dependency inventory")
  endif()
  foreach(dependency IN LISTS imports)
    if(NOT dependency MATCHES "\n\t(/usr/lib/|/System/Library/)")
      message(FATAL_ERROR "FFmpeg requires an unpackaged library: ${dependency}")
    endif()
  endforeach()
  execute_process(COMMAND codesign --force --sign - "${executable}" COMMAND_ERROR_IS_FATAL ANY)
else()
  execute_process(COMMAND "${CMAKE_DUMPBIN}" /NOLOGO /DEPENDENTS "${executable}" OUTPUT_VARIABLE imports
    COMMAND_ERROR_IS_FATAL ANY)
  file(READ "${CMAKE_CURRENT_SOURCE_DIR}/../../../build_support/windows-system-libraries.json" allowed)
  string(TOLOWER "${imports}" imports)
  string(REGEX MATCHALL "[a-z0-9_.+-]+\\.(dll|drv)" dependencies "${imports}")
  if(NOT dependencies)
    message(FATAL_ERROR "Media executable has no system dependency inventory")
  endif()
  foreach(name IN LISTS dependencies)
    string(FIND "${allowed}" "\"${name}\"" listed)
    if(NOT name MATCHES "^(api-ms-|ext-ms-)" AND (listed EQUAL -1 OR NOT EXISTS "$ENV{SYSTEMROOT}/System32/${name}"))
      message(FATAL_ERROR "FFmpeg requires an unpackaged library: ${name}")
    endif()
  endforeach()
endif()
string(JSON version GET "${source}" version)
execute_process(COMMAND "${executable}" -version OUTPUT_VARIABLE actual COMMAND_ERROR_IS_FATAL ANY)
if(NOT actual MATCHES "ffmpeg version ${version}")
  message(FATAL_ERROR "FFmpeg version differs from its source record")
endif()
execute_process(COMMAND "${executable}" -hide_banner -buildconf OUTPUT_VARIABLE configuration
  ERROR_VARIABLE configuration_errors COMMAND_ERROR_IS_FATAL ANY)
file(WRITE "${CMAKE_INSTALL_PREFIX}/build-evidence/ffmpeg-version.txt" "${actual}")
file(WRITE "${CMAKE_INSTALL_PREFIX}/build-evidence/ffmpeg-buildconf.txt" "${configuration}${configuration_errors}")
if("${configuration}${configuration_errors}" MATCHES "--enable-nonfree")
  message(FATAL_ERROR "Core FFmpeg must remain redistributable")
endif()
execute_process(COMMAND "${executable}" -hide_banner -filters OUTPUT_VARIABLE filters COMMAND_ERROR_IS_FATAL ANY)
foreach(filter IN ITEMS subtitles astats loudnorm concat xfade acrossfade amix afade fade
    trim atrim setpts asetpts scale pad crop format fps settb aresample aformat anullsrc apad adelay anull null)
  if(NOT filters MATCHES " ${filter} ")
    message(FATAL_ERROR "FFmpeg lacks required filter: ${filter}")
  endif()
endforeach()
if(WIN32)
  set(encoder h264_mf)
else()
  set(encoder h264_videotoolbox)
endif()
execute_process(COMMAND "${executable}" -hide_banner -encoders OUTPUT_VARIABLE encoders COMMAND_ERROR_IS_FATAL ANY)
if(NOT encoders MATCHES " ${encoder} ")
  message(FATAL_ERROR "FFmpeg lacks required H.264 encoder: ${encoder}")
endif()
# Audiobook production and resource export name these encoders directly.
foreach(required IN ITEMS libx264 aac libmp3lame flac pcm_s16le pcm_f32le)
  if(NOT encoders MATCHES " ${required} ")
    message(FATAL_ERROR "FFmpeg lacks required encoder: ${required}")
  endif()
endforeach()
execute_process(COMMAND "${executable}" -hide_banner -loglevel error -nostdin -y
  -f lavfi -i color=c=black:s=96x64:d=0.3 -vf "scale=64:48,crop=48:32,format=yuv420p"
  -c:v libx264 "${CMAKE_BINARY_DIR}/smoke.mp4" COMMAND_ERROR_IS_FATAL ANY)
execute_process(COMMAND "${executable}" -hide_banner -loglevel error -nostdin -y
  -f lavfi -i sine=frequency=440:duration=0.3 -af "aresample=16000,atrim=duration=0.2"
  -c:a pcm_s16le "${CMAKE_BINARY_DIR}/smoke.wav" COMMAND_ERROR_IS_FATAL ANY)
set(scratch "${CMAKE_BINARY_DIR}/media-smoke-中文")
file(MAKE_DIRECTORY "${scratch}")
file(WRITE "${scratch}/captions.ass" "[Script Info]\nScriptType: v4.00+\nPlayResX: 64\nPlayResY: 64\n[V4+ Styles]\nFormat: Name, Fontname, Fontsize, PrimaryColour, Alignment\nStyle: Default,Arial,12,&H00FFFFFF,2\n[Events]\nFormat: Layer, Start, End, Style, Text\nDialogue: 0,0:00:00.00,0:00:00.20,Default,Qualification\n")
execute_process(COMMAND "${executable}" -hide_banner -loglevel error -nostdin
  -f lavfi -i color=c=black:s=64x64:r=10:d=0.2
  -f lavfi -i sine=frequency=440:sample_rate=48000:duration=0.2
  -af loudnorm=I=-18:TP=-1:LRA=11 -c:v libx264 -c:a aac -f null -
  WORKING_DIRECTORY "${scratch}" COMMAND_ERROR_IS_FATAL ANY)
execute_process(COMMAND "${executable}" -hide_banner -loglevel error -nostdin
  -f lavfi -i color=s=64x64:r=10:d=0.1 -f lavfi -i color=s=64x64:r=10:d=0.1
  -filter_complex "[0:v][1:v]concat=n=2:v=1:a=0,subtitles=captions.ass[v]"
  -map "[v]" -c:v libx264 -f null - WORKING_DIRECTORY "${scratch}" COMMAND_ERROR_IS_FATAL ANY)
execute_process(COMMAND "${executable}" -hide_banner -loglevel error -nostdin
  -f lavfi -i sine=duration=0.1 -af astats -c:a pcm_s16le -f null -
  WORKING_DIRECTORY "${scratch}" COMMAND_ERROR_IS_FATAL ANY)
file(REMOVE_RECURSE "${scratch}")
