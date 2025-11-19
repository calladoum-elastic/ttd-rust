#define CATCH_CONFIG_MAIN

#include <windows.h>

#include <catch2/catch_test_macros.hpp>
#include <filesystem>

#include "ttd_ffi.hpp"

#define NS "Basic"

std::filesystem::path
GetTemp()
{
    wchar_t tempPath[MAX_PATH] {};
    ::GetTempPathW(MAX_PATH, tempPath);
    return std::filesystem::path(tempPath);
}


TEST_CASE("TTD FFI Tests", "[" NS "]")
{
    SECTION("Load trace")
    {
        REQUIRE(TTD_FFI::Replay::Initialize() == 0);

        const auto invalid_path = GetTemp() / "iDontExist";
        const auto valid_path   = GetTemp() / "test.run";
        REQUIRE(TTD_FFI::Replay::Load((const u16*)invalid_path.c_str()) == -1);
        REQUIRE(TTD_FFI::Replay::Load((const u16*)valid_path.c_str()) == 0);
    }

    SECTION("Navigate trace")
    {
        REQUIRE(TTD_FFI::Replay::Initialize() == 0);
        const auto valid_path = GetTemp() / "test.run";
        REQUIRE(TTD_FFI::Replay::Load((const u16*)valid_path.c_str()) == 0);

        auto const CurPos = TTD_FFI::Replay::GetPosition();
        REQUIRE(CurPos != TTD::Replay::Position::Invalid);

        TTD_FFI::Replay::SetPosition(CurPos + 1);
        REQUIRE(TTD_FFI::Replay::GetPosition() == CurPos + 1);
    }
}
