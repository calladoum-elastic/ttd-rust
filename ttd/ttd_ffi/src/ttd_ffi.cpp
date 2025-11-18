// clang-format off
#include "ttd_ffi.hpp"

#include <windows.h>

#include <wrl.h>

#include <cstdio>
#include <filesystem>
#include <string>
#include <type_traits>
#include <array>
#include <atomic>
#include <mutex>

#include <TTD/IReplayEngineStl.h>

#include <TTD/TTDLiveRecorder.h>
// clang-format on


#ifdef _DEBUG
#define dbg(fmt, ...) ::wprintf("[*] " fmt L"\n", __VA_ARGS__)
#define ok(fmt, ...) ::wprintf("[+] " fmt L"\n", __VA_ARGS__)
#define err(fmt, ...) ::wprintf("[-] " fmt L"\n", __VA_ARGS__)
#else
#define dbg(fmt, ...)
#define ok(fmt, ...)
#define err(fmt, ...)
#endif // _DEBUG


static TTD::Replay::UniqueReplayEngine g_Engine {nullptr};
static TTD::Replay::UniqueCursor g_Cursor {nullptr};

i32
TTD_FFI::Replay::Initialize()
{
    dbg(L"Initializing %S...", TTD_FFI::LibraryBanner);

    if ( g_Engine )
    {
        ok(L"already initialized");
        return 0;
    }

    ok(L"Initialized %S...", TTD_FFI::LibraryBanner);

    auto [engine, result] = TTD::Replay::MakeReplayEngine();
    if ( result != 0 || !engine )
    {
        err(L"MakeReplayEngine() failed: retcode=%x", result);
        return -1;
    }

    g_Engine = std::move(engine);

    return 0;
}


i32
TTD_FFI::Replay::Load(const u16* trace)
{
    // const std::filesystem::path tracePath {(const i16*)trace};
    // if ( !std::filesystem::exists(tracePath) )
    // {
    //     err(L"File %S doesn't exist", tracePath.c_str());
    //     return -1;
    // }

    // dbg(L"Loading trace %s", tracePath.wstring().c_str());
    // const std::wstring ws = tracePath.wstring();
    // LPWSTR ptr            = (LPWSTR)tracePath.wstring().c_str();
    if ( !g_Engine->Initialize((PCWSTR)trace) )
    {
        err(L"Initialize('%s') failed", trace);
        return -1;
    }

    ok(L"Loaded %s...", trace);

    dbg(L"creating cursor");
    TTD::Replay::UniqueCursor cursor(g_Engine->NewCursor());
    dbg(L"setting cursor to start of trace");
    cursor->SetPosition(g_Engine->GetLifetime().Min);

    g_Cursor = std::move(cursor);
    return 0;
}

int32_t
TTD_FFI::Replay::Reset()
{
    if ( !g_Engine )
    {
        err(L"not initialized");
        return 1;
    }

    dbg(L"deallocating engine");
    g_Engine = nullptr;
    g_Cursor = nullptr;

    return 0;
}


TTD::SystemInfo
TTD_FFI::Replay::GetSystemInfo()
{
    return g_Engine->GetSystemInfo();
}


u32
TTD_FFI::Replay::BuildIndex()
{
    auto progress_cb = [](void const* pCallerContext, TTD::Replay::IndexBuildProgressType const* pProgressData) {};

    return std::underlying_type_t<TTD::Replay::IndexStatus>(
        g_Engine->BuildIndex(progress_cb, nullptr, TTD::Replay::IndexBuildFlags::None));
}


TTD::Replay::ICursorView::ReplayResult
TTD_FFI::Replay::ReplayForward()
{
    return ReplayForward(TTD::Replay::Position::Max);
}


TTD::Replay::ICursorView::ReplayResult
TTD_FFI::Replay::ReplayForward(TTD::Replay::Position limit)
{
#ifdef _DEBUG
    wchar_t from[1000] {};
    wchar_t to[1000] {};

    const auto cur = TTD_FFI::Replay::GetPosition();
    TTD::Replay::PositionToString(cur, from, _countof(from));
    TTD::Replay::PositionToString(limit, to, _countof(to));

    dbg(L"Forward replaying from %s to %s", from, to);
#endif // _DEBUG

    return g_Cursor->ReplayForward(limit);
}


TTD::Replay::ICursorView::ReplayResult
TTD_FFI::Replay::ReplayBackward()
{
    return ReplayBackward(TTD::Replay::Position::Min);
}


TTD::Replay::ICursorView::ReplayResult
TTD_FFI::Replay::ReplayBackward(TTD::Replay::Position limit)
{
#ifdef _DEBUG
    wchar_t from[1000] {};
    wchar_t to[1000] {};

    const auto cur = TTD_FFI::Replay::GetPosition();
    TTD::Replay::PositionToString(cur, from, _countof(from));
    TTD::Replay::PositionToString(limit, to, _countof(to));

    dbg(L"Backward replaying from %s to %s", from, to);
#endif // _DEBUG

    return g_Cursor->ReplayBackward(limit);
}


void
TTD_FFI::Replay::SetPosition(TTD::Replay::Position const& pos)
{
    dbg(L"Setting position %x:%x", pos.Sequence, pos.Steps);
    return g_Cursor->SetPosition(pos);
}

TTD::Replay::Position
TTD_FFI::Replay::GetPosition()
{
    return g_Cursor->GetPosition();
}

i32
TTD_FFI::Replay::QueryMemoryBuffer(u64 address, u8* buf, usize bufsz)
{
    dbg(L"QueryMemoryBuffer(addr=%llx ,buf=%p, bufsz=%d)", address, buf, bufsz);
    const TTD::Replay::MemoryBuffer res = g_Cursor->QueryMemoryBuffer(
        TTD::GuestAddress {address},
        TTD::BufferView {buf, bufsz},
        TTD::Replay::QueryMemoryPolicy::Default);

    if ( res.Memory.IsNull() || !res.Memory.IsValid() )
    {
        return -1;
    }

    return 0;
}

TTD::Replay::ThreadInfo const&
TTD_FFI::Replay::GetThreadInfo()
{
    return g_Cursor->GetThreadInfo();
}

TTD::Replay::Position
TTD_FFI::Replay::GetPreviousPosition()
{
    return g_Cursor->GetPreviousPosition();
}

u64
TTD_FFI::Replay::GetTebAddress()
{
    return (u64)g_Cursor->GetTebAddress();
}

u64
TTD_FFI::Replay::GetProgramCounter()
{
    return (u64)g_Cursor->GetProgramCounter();
}

u64
TTD_FFI::Replay::GetStackPointer()
{
    return (u64)g_Cursor->GetStackPointer();
}

u64
TTD_FFI::Replay::GetFramePointer()
{
    return (u64)g_Cursor->GetFramePointer();
}

X86_NT5_CONTEXT
TTD_FFI::Replay::GetX86RegisterContext()
{
    const auto ctx = reinterpret_cast<X86_NT5_CONTEXT*>(g_Cursor->GetCrossPlatformContext().Data);
    return *ctx;
}

AVX_EXTENDED_CONTEXT
TTD_FFI::Replay::GetX86ExtendedRegisterContext()
{
    const auto ctx = reinterpret_cast<AVX_EXTENDED_CONTEXT*>(g_Cursor->GetCrossPlatformContext().Data);
    return *ctx;
}

AMD64_CONTEXT
TTD_FFI::Replay::GetX64RegisterContext()
{
    const auto ctx = reinterpret_cast<AMD64_CONTEXT*>(g_Cursor->GetCrossPlatformContext().Data);
    return *ctx;
}


AVX_EXTENDED_CONTEXT
TTD_FFI::Replay::GetX64ExtendedRegisterContext()
{
    const auto ctx = reinterpret_cast<AVX_EXTENDED_CONTEXT*>(g_Cursor->GetCrossPlatformContext().Data);
    return *ctx;
}

void
TTD_FFI::Replay::SetReplayFlags(TTD::Replay::ReplayFlags flags)
{
    return g_Cursor->SetReplayFlags(flags);
}

TTD::Replay::ReplayFlags
TTD_FFI::Replay::GetReplayFlags()
{
    return g_Cursor->GetReplayFlags();
}

void
TTD_FFI::Replay::SetReplayProgressCallback(TTD::Replay::ICursorView::ReplayProgressCallback* cb, uptr context)
{
    g_Cursor->SetReplayProgressCallback(cb, context);
}

void
TTD_FFI::Replay::SetRegisterChangedCallback(TTD::Replay::ICursorView::RegisterChangedCallback* cb, uptr context)
{
    g_Cursor->SetRegisterChangedCallback(cb, context);
}


bool
TTD_FFI::Replay::AddMemoryWatchpoint(_In_ TTD::Replay::MemoryWatchpointData const& WatchPoint)
{
    return g_Cursor->AddMemoryWatchpoint(WatchPoint);
}

bool
TTD_FFI::Replay::RemoveMemoryWatchpoint(_In_ TTD::Replay::MemoryWatchpointData const& WatchPoint)
{
    return g_Cursor->RemoveMemoryWatchpoint(WatchPoint);
}

bool
TTD_FFI::Replay::AddPositionWatchpoint(_In_ TTD::Replay::PositionWatchpointData const& WatchPoint)
{
    return g_Cursor->AddPositionWatchpoint(WatchPoint);
}

bool
TTD_FFI::Replay::RemovePositionWatchpoint(_In_ TTD::Replay::PositionWatchpointData const& WatchPoint)
{
    return g_Cursor->RemovePositionWatchpoint(WatchPoint);
}

size_t
TTD_FFI::Replay::GetModuleCount()
{
    return g_Engine->GetModuleCount();
}

TTD::Replay::Module const*
TTD_FFI::Replay::GetModuleList()
{
    return g_Engine->GetModuleList();
}

size_t
TTD_FFI::Replay::GetModuleInstanceCount()
{
    return g_Engine->GetModuleInstanceCount();
}

TTD::Replay::ModuleInstance const*
TTD_FFI::Replay::GetModuleInstanceList()
{
    return g_Engine->GetModuleInstanceList();
}

size_t
TTD_FFI::Replay::GetThreadCount()
{
    return g_Engine->GetThreadCount();
}

TTD::Replay::ThreadInfo const*
TTD_FFI::Replay::GetThreadList()
{
    return g_Engine->GetThreadList();
}

size_t
TTD_FFI::Replay::GetModuleLoadedEventCount()
{
    return g_Engine->GetModuleLoadedEventCount();
}

TTD::Replay::ModuleLoadedEvent const*
TTD_FFI::Replay::GetModuleLoadedEventList()
{
    return g_Engine->GetModuleLoadedEventList();
}

size_t
TTD_FFI::Replay::GetModuleUnloadedEventCount()
{
    return g_Engine->GetModuleUnloadedEventCount();
}

TTD::Replay::ModuleUnloadedEvent const*
TTD_FFI::Replay::GetModuleUnloadedEventList()
{
    return g_Engine->GetModuleUnloadedEventList();
}

size_t
TTD_FFI::Replay::GetExceptionEventCount()
{
    return g_Engine->GetExceptionEventCount();
}

TTD::Replay::ExceptionEvent const*
TTD_FFI::Replay::GetExceptionEventList()
{
    return g_Engine->GetExceptionEventList();
}
