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
#include <ranges>

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

static std::array<TTD::Replay::UniqueReplayEngine, 256> g_Engines {};
static std::mutex g_EnginesMutex {};

static std::array<TTD::Replay::UniqueCursor, 256> g_Cursors {};
static std::mutex g_CursorsMutex {};

#pragma region TTD_FFI::Replay::ReplayEngine

#define GetEngineSafe()                                                                                                \
    std::lock_guard _lock(g_EnginesMutex);                                                                             \
    if ( this->m_Index < 0 || this->m_Index > g_Engines.size() )                                                       \
    {                                                                                                                  \
        throw "Corrupted state";                                                                                       \
    }                                                                                                                  \
    auto& engine = g_Engines.at(this->m_Index);                                                                        \
    if ( !engine )                                                                                                     \
    {                                                                                                                  \
        throw "Corrupted state";                                                                                       \
    }

TTD_FFI::Replay::ReplayEngine::ReplayEngine() : m_Index {-1}
{
}

TTD_FFI::Replay::ReplayEngine::~ReplayEngine()
{
    dbg(L"destroying ReplayEngine (%ld)", this->m_Index);
    this->Reset();
    return;
}

i32
TTD_FFI::Replay::ReplayEngine::Initialize()
{
    std::lock_guard _lock(g_EnginesMutex);
    if ( this->m_Index >= 0 )
    {
        return this->m_Index;
    }

    for ( auto [idx, cur& ] : std::views::enumerate(g_Engines) )
    {
        if ( cur )
        {
            continue;
        }

        auto [engine, result] = TTD::Replay::MakeReplayEngine();
        if ( result != 0 || !engine )
        {
            err(L"MakeReplayEngine() failed: retcode=%x", result);
            break;
        }

        cur           = std::move(engine);
        this->m_Index = idx;
        dbg("Allocated new replay engine at Idx=%d", this->m_Index);
        break;
    }

    return this->m_Index;
}

i32
TTD_FFI::Replay::ReplayEngine::Load(const u16* trace) const
{
    GetEngineSafe();

    const std::filesystem::path tracePath {(const wchar_t*)trace};
    if ( !std::filesystem::exists(tracePath) )
    {
        err(L"File %S doesn't exist", tracePath.string().c_str());
        return -2;
    }

    dbg(L"Loading trace %s", tracePath.wstring().c_str());
    if ( !engine->Initialize(tracePath.wstring().c_str()) )
    {
        err(L"Initialize('%S') failed", tracePath.string().c_str());
        return -1;
    }

    ok(L"Loaded %s...", tracePath.wstring().c_str());
    return 0;
}

i32
TTD_FFI::Replay::ReplayEngine::Reset()
{
    GetEngineSafe();
    dbg(L"deallocating engine");
    engine  = nullptr;
    m_Index = -1;
    return 0;
}

TTD::SystemInfo const&
TTD_FFI::Replay::ReplayEngine::GetSystemInfo() const
{
    GetEngineSafe();
    return engine->GetSystemInfo();
}


u32
TTD_FFI::Replay::ReplayEngine::BuildIndex() const
{
    GetEngineSafe();
    auto progress_cb = [](void const* pCallerContext, TTD::Replay::IndexBuildProgressType const* pProgressData) {};

    return std::underlying_type_t<TTD::Replay::IndexStatus>(
        engine->BuildIndex(progress_cb, nullptr, TTD::Replay::IndexBuildFlags::None));
}

size_t
TTD_FFI::Replay::ReplayEngine::GetModuleCount() const
{
    GetEngineSafe();
    return engine->GetModuleCount();
}

TTD::Replay::Module const*
TTD_FFI::Replay::ReplayEngine::GetModuleList() const
{
    GetEngineSafe();
    return engine->GetModuleList();
}

size_t
TTD_FFI::Replay::ReplayEngine::GetModuleInstanceCount() const
{
    GetEngineSafe();
    return engine->GetModuleInstanceCount();
}

TTD::Replay::ModuleInstance const*
TTD_FFI::Replay::ReplayEngine::GetModuleInstanceList() const
{
    GetEngineSafe();
    return engine->GetModuleInstanceList();
}

size_t
TTD_FFI::Replay::ReplayEngine::GetThreadCount() const
{
    GetEngineSafe();
    return engine->GetThreadCount();
}

TTD::Replay::ThreadInfo const*
TTD_FFI::Replay::ReplayEngine::GetThreadList() const
{
    GetEngineSafe();
    return engine->GetThreadList();
}

size_t
TTD_FFI::Replay::ReplayEngine::GetModuleLoadedEventCount() const
{
    GetEngineSafe();
    return engine->GetModuleLoadedEventCount();
}

TTD::Replay::ModuleLoadedEvent const*
TTD_FFI::Replay::ReplayEngine::GetModuleLoadedEventList() const
{
    GetEngineSafe();
    return engine->GetModuleLoadedEventList();
}

size_t
TTD_FFI::Replay::ReplayEngine::GetModuleUnloadedEventCount() const
{
    GetEngineSafe();
    return engine->GetModuleUnloadedEventCount();
}

TTD::Replay::ModuleUnloadedEvent const*
TTD_FFI::Replay::ReplayEngine::GetModuleUnloadedEventList() const
{
    GetEngineSafe();
    return engine->GetModuleUnloadedEventList();
}

size_t
TTD_FFI::Replay::ReplayEngine::GetExceptionEventCount() const
{
    GetEngineSafe();
    return engine->GetExceptionEventCount();
}

TTD::Replay::ExceptionEvent const*
TTD_FFI::Replay::ReplayEngine::GetExceptionEventList() const
{
    GetEngineSafe();
    return engine->GetExceptionEventList();
}

#pragma endregion TTD_FFI::Replay::ReplayEngine

#pragma region TTD_FFI::Replay::ReplayCursor

#define GetCursorSafe()                                                                                                \
    std::lock_guard _lock(g_CursorsMutex);                                                                             \
    dbg(L"cursor_idx=%x", this->m_Index);                                                                              \
    if ( this->m_Index < 0 || this->m_Index > g_Cursors.size() )                                                       \
    {                                                                                                                  \
        throw "Incorrect index";                                                                                       \
    }                                                                                                                  \
    auto& cursor = g_Cursors[this->m_Index];                                                                           \
    if ( !cursor )                                                                                                     \
    {                                                                                                                  \
        throw "Corrupted state";                                                                                       \
    }

TTD_FFI::Replay::ReplayCursor::ReplayCursor() : m_Index {-1}, m_EngineIndex {-1}
{
}

TTD_FFI::Replay::ReplayCursor::~ReplayCursor()
{
    dbg("destroying ReplayCursor (%ld)", this->m_Index);
    this->Reset();
}

i32
TTD_FFI::Replay::ReplayCursor::Initialize(i32 EngineIndex)
{
    std::lock_guard _lock(g_EnginesMutex);
    std::lock_guard _lock2(g_CursorsMutex);

    if ( this->m_Index >= 0 )
    {
        return this->m_Index;
    }

    this->m_Index = -1;
    auto& engine  = g_Engines[EngineIndex];
    if ( !engine )
    {
        return -1;
    }

    for ( auto [idx, cur& ] : std::views::enumerate(g_Cursors) )
    {
        if ( cur )
        {
            continue;
        }

        auto cursor = TTD::Replay::UniqueCursor(engine->NewCursor());
        if ( !cursor )
        {
            err(L"NewCursor(%d) failed", EngineIndex);
            break;
        }

        cur                 = std::move(cursor);
        this->m_Index       = idx;
        this->m_EngineIndex = EngineIndex;
        dbg("Allocated new replay cursor at Idx=%d for engine Idx=%d", this->m_Index, this->m_EngineIndex);
        break;
    }

    return this->m_Index;
}

i32
TTD_FFI::Replay::ReplayCursor::Reset()
{
    std::lock_guard _lock(g_CursorsMutex);
    auto& cursor = g_Cursors[m_Index];
    if ( !cursor )
    {
        err(L"not initialized");
        return 1;
    }

    dbg(L"deallocating engine");
    cursor  = nullptr;
    m_Index = -1;
    return 0;
}

TTD::Replay::ICursorView::ReplayResult
TTD_FFI::Replay::ReplayCursor::ReplayForward(TTD::Replay::Position const& limit)
{
    GetCursorSafe();

#ifdef _DEBUG
    wchar_t from[100] {};
    wchar_t to[100] {};

    const auto cur = cursor->GetPosition();
    TTD::Replay::PositionToString(cur, from, _countof(from));
    TTD::Replay::PositionToString(limit, to, _countof(to));
    dbg(L"Forward replaying from %s to %s", from, to);
#endif // _DEBUG

    return cursor->ReplayForward(limit);
}


TTD::Replay::ICursorView::ReplayResult
TTD_FFI::Replay::ReplayCursor::ReplayBackward(TTD::Replay::Position limit)
{
    GetCursorSafe();

#ifdef _DEBUG
    wchar_t from[100] {};
    wchar_t to[100] {};

    const auto cur = cursor->GetPosition();
    TTD::Replay::PositionToString(cur, from, _countof(from));
    TTD::Replay::PositionToString(limit, to, _countof(to));
    dbg(L"Backward replaying from %s to %s", from, to);
#endif // _DEBUG

    return cursor->ReplayBackward(limit);
}

i32
TTD_FFI::Replay::ReplayCursor::QueryMemoryBuffer(u64 address, u8* buf, usize bufsz) const
{
    GetCursorSafe();

    dbg(L"QueryMemoryBuffer(addr=%llx ,buf=%p, bufsz=%lu)", address, buf, bufsz);
    const TTD::Replay::MemoryBuffer res = cursor->QueryMemoryBuffer(
        TTD::GuestAddress {address},
        TTD::BufferView {buf, bufsz},
        TTD::Replay::QueryMemoryPolicy::Default);

    if ( res.Memory.IsNull() || !res.Memory.IsValid() )
    {
        return -1;
    }

    return 0;
}

void
TTD_FFI::Replay::ReplayCursor::SetPosition(TTD::Replay::Position const& pos)
{
    dbg(L"Setting position %lx:%lx", (uint64_t)pos.Sequence, (uint64_t)pos.Steps);
    GetCursorSafe();
    return cursor->SetPosition(pos);
}

TTD::Replay::Position
TTD_FFI::Replay::ReplayCursor::GetPosition() const
{
    GetCursorSafe();
    return cursor->GetPosition();
}

TTD::Replay::Position
TTD_FFI::Replay::ReplayCursor::GetPreviousPosition() const
{
    GetCursorSafe();
    return cursor->GetPreviousPosition();
}

TTD::Replay::ThreadInfo const&
TTD_FFI::Replay::ReplayCursor::GetThreadInfo() const
{
    GetCursorSafe();
    return cursor->GetThreadInfo();
}

u64
TTD_FFI::Replay::ReplayCursor::GetTebAddress() const
{
    GetCursorSafe();
    return (u64)cursor->GetTebAddress();
}

u64
TTD_FFI::Replay::ReplayCursor::GetProgramCounter() const
{
    GetCursorSafe();
    return (u64)cursor->GetProgramCounter();
}

u64
TTD_FFI::Replay::ReplayCursor::GetStackPointer() const
{
    GetCursorSafe();
    return (u64)cursor->GetStackPointer();
}

u64
TTD_FFI::Replay::ReplayCursor::GetFramePointer() const
{
    GetCursorSafe();
    return (u64)cursor->GetFramePointer();
}

X86_NT5_CONTEXT*
TTD_FFI::Replay::ReplayCursor::GetX86RegisterContext() const
{
    GetCursorSafe();
    return reinterpret_cast<X86_NT5_CONTEXT*>(cursor->GetCrossPlatformContext().Data);
}

AVX_EXTENDED_CONTEXT*
TTD_FFI::Replay::ReplayCursor::GetX86ExtendedRegisterContext() const
{
    GetCursorSafe();
    return reinterpret_cast<AVX_EXTENDED_CONTEXT*>(cursor->GetCrossPlatformContext().Data);
}

AMD64_CONTEXT*
TTD_FFI::Replay::ReplayCursor::GetX64RegisterContext() const
{
    GetCursorSafe();
    return reinterpret_cast<AMD64_CONTEXT*>(cursor->GetCrossPlatformContext().Data);
}


AVX_EXTENDED_CONTEXT*
TTD_FFI::Replay::ReplayCursor::GetX64ExtendedRegisterContext() const
{
    GetCursorSafe();
    return reinterpret_cast<AVX_EXTENDED_CONTEXT*>(cursor->GetCrossPlatformContext().Data);
}

void
TTD_FFI::Replay::ReplayCursor::SetReplayFlags(TTD::Replay::ReplayFlags flags)
{
    GetCursorSafe();
    return cursor->SetReplayFlags(flags);
}

TTD::Replay::ReplayFlags
TTD_FFI::Replay::ReplayCursor::GetReplayFlags() const
{
    GetCursorSafe();
    return cursor->GetReplayFlags();
}

bool
TTD_FFI::Replay::ReplayCursor::AddMemoryWatchpoint(_In_ TTD::Replay::MemoryWatchpointData const& WatchPoint)
{
    GetCursorSafe();
    return cursor->AddMemoryWatchpoint(WatchPoint);
}

bool
TTD_FFI::Replay::ReplayCursor::RemoveMemoryWatchpoint(_In_ TTD::Replay::MemoryWatchpointData const& WatchPoint)
{
    GetCursorSafe();
    return cursor->RemoveMemoryWatchpoint(WatchPoint);
}

bool
TTD_FFI::Replay::ReplayCursor::AddPositionWatchpoint(_In_ TTD::Replay::PositionWatchpointData const& WatchPoint)
{
    GetCursorSafe();
    return cursor->AddPositionWatchpoint(WatchPoint);
}

bool
TTD_FFI::Replay::ReplayCursor::RemovePositionWatchpoint(_In_ TTD::Replay::PositionWatchpointData const& WatchPoint)
{
    GetCursorSafe();
    return cursor->RemovePositionWatchpoint(WatchPoint);
}

void
TTD_FFI::Replay::ReplayCursor::SetReplayProgressCallback(
    TTD::Replay::ICursorView::ReplayProgressCallback* cb,
    uptr context)
{
    GetCursorSafe();
    cursor->SetReplayProgressCallback(cb, context);
}

void
TTD_FFI::Replay::ReplayCursor::SetRegisterChangedCallback(
    TTD::Replay::ICursorView::RegisterChangedCallback* cb,
    uptr context)
{
    GetCursorSafe();
    cursor->SetRegisterChangedCallback(cb, context);
}

#pragma endregion TTD_FFI::Replay::ReplayCursor
