#define CATCH_CONFIG_MAIN

#include <windows.h>

#include <catch2/catch_test_macros.hpp>
#include <filesystem>

#define private public
#define protected public

#include "ttd_ffi.hpp"

#define NS "Basic"

static std::filesystem::path
GetTemp()
{
    wchar_t tempPath[MAX_PATH] {};
    ::GetTempPathW(MAX_PATH, tempPath);
    return std::filesystem::path(tempPath);
}

struct ReplayEngineTestClass
{
    TTD_FFI::Replay::ReplayEngine Engine;
    ReplayEngineTestClass() : Engine()
    {
        const auto valid_path = GetTemp() / L"test.run";
        this->Engine.Load((const u16*)valid_path.c_str());
    }
};


TEST_CASE_METHOD(ReplayEngineTestClass, "Basic Test", "Navigate trace")
{
    auto Cursor = TTD_FFI::Replay::ReplayCursor(this->Engine.Index());
    REQUIRE(0 <= Cursor.Index());
    REQUIRE(Cursor.Index() < TTD_FFI::Replay::MAX_CURSOR);
    REQUIRE(0 <= Cursor.EngineIndex());
    REQUIRE(Cursor.EngineIndex() < TTD_FFI::Replay::MAX_ENGINE);

    REQUIRE(Cursor.GetPosition() == TTD::Replay::Position::Invalid);

    // Cursor.SetPosition(Engine.GetLifetime().Min);
    // REQUIRE(Cursor.GetPosition() == Engine.GetLifetime().Min);
}
