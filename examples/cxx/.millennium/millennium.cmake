file(WRITE ${CMAKE_BINARY_DIR}/_millennium.cc "")

add_library(millennium STATIC ${CMAKE_BINARY_DIR}/_millennium.cc)
target_link_libraries(millennium PUBLIC millenniumcxx)

if(${CMAKE_BUILD_TYPE} MATCHES "(Release)|(MinSizeRel)|(RelWithDebInfo)")
	set(_MILLENNIUM_CARGO_TARGET "release")
else()
	set(_MILLENNIUM_CARGO_TARGET "debug")
endif()

set(_MILLENNIUM_LINK_LIBRARY_DIR "${CMAKE_CURRENT_LIST_DIR}/target/${_MILLENNIUM_CARGO_TARGET}")

if(${_MILLENNIUM_CARGO_TARGET} MATCHES "release")
	add_custom_target(millennium_cargo
		COMMAND cargo build --release
		WORKING_DIRECTORY ${CMAKE_CURRENT_LIST_DIR}
	)
else()
	add_custom_target(millennium_cargo
		COMMAND cargo build
		WORKING_DIRECTORY ${CMAKE_CURRENT_LIST_DIR}
	)
endif()

target_include_directories(millennium PUBLIC ${CMAKE_CURRENT_LIST_DIR}/target)
add_dependencies(millennium millennium_cargo)

if(WIN32)
	include(FetchContent)
	FetchContent_Declare(
		millennium_webview2
		URL https://www.nuget.org/api/v2/package/Microsoft.Web.WebView2/1.0.961.33
	)
	FetchContent_Populate(millennium_webview2)

	if(CMAKE_SYSTEM_PROCESSOR MATCHES "x86|X86")
		target_link_directories(millennium PUBLIC ${millennium_webview2_SOURCE_DIR}/build/native/x86)
	elseif(CMAKE_SYSTEM_PROCESSOR MATCHES "amd64|AMD64")
		target_link_directories(millennium PUBLIC ${millennium_webview2_SOURCE_DIR}/build/native/x64)
	elseif(CMAKE_SYSTEM_PROCESSOR MATCHES "arm64|ARM64")
		target_link_directories(millennium PUBLIC ${millennium_webview2_SOURCE_DIR}/build/native/arm64)
	else()
		message(FATAL_ERROR "Unsupported processor: ${CMAKE_SYSTEM_PROCESSOR}")
	endif()

	set(_MILLENNIUM_WIN32_LIBS WebView2LoaderStatic imm32 ws2_32 userenv uxtheme comctl32 dwmapi bcrypt)
	target_link_libraries(millennium PUBLIC ${_MILLENNIUM_WIN32_LIBS})

	message(STATUS "webview2: " ${millennium_webview2_SOURCE_DIR})
endif()

target_link_directories(millennium PUBLIC ${_MILLENNIUM_LINK_LIBRARY_DIR})
