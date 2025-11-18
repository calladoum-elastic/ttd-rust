#define CATCH_CONFIG_MAIN

#include <catch2/catch_test_macros.hpp>

#include "ttd_ffi.hpp"

#define NS "Basic"

#define TEST_TRACE_PATH_INVALID L"c:\\users\\chris\\documents\\AAAAAAAAAAAA.run"
#define TEST_TRACE_PATH L"c:\\users\\chris\\documents\\notepad03.run"

TEST_CASE("TTD FFI Tests", "[" NS "]")
{
    SECTION("Load trace")
    {
        REQUIRE(TTD_FFI::Replay::Initialize() == 0);
        REQUIRE(TTD_FFI::Replay::Load((const u16*)TEST_TRACE_PATH_INVALID) == -1);
        REQUIRE(TTD_FFI::Replay::Load((const u16*)TEST_TRACE_PATH) == 0);
    }
}
