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

#ifdef _LIBTTD_VERBOSE_OUTPUT
#define ok(fmt, ...) ::wprintf("[+] " fmt L"\n", __VA_ARGS__)
#define err(fmt, ...) ::wprintf("[-] " fmt L"\n", __VA_ARGS__)
#ifdef _DEBUG
#define dbg(fmt, ...) ::wprintf("[*] %S " fmt L"\n", __FUNCTION__, __VA_ARGS__)
#else
#define dbg(fmt, ...)
#endif // _DEBUG
#else
#define dbg(fmt, ...)
#define ok(fmt, ...)
#define err(fmt, ...)
#endif // _LIBTTD_VERBOSE_OUTPUT


#pragma region TTD_FFI::Replay::ReplayEngine


TTD_FFI::Replay::ReplayEngine::ReplayEngine()
{
    auto [eng, res] = TTD::Replay::MakeReplayEngine();
    if ( res != 0 || !eng )
    {
        throw "CRITICAL - Cannot create replay engine";
    }

    this->m_Engine = std::move(eng);
}

TTD_FFI::Replay::ReplayEngine::~ReplayEngine()
{
    this->m_Engine = nullptr;
}

const uptr
TTD_FFI::Replay::ReplayEngine::NewCursor() const
{
    auto cur = this->m_Engine->NewCursor();
    if ( !cur )
    {
        err("CRITICAL - Cannot create cursor from ReplayEngine");
        return 0;
    }

    return (uptr)cur;
}

i32
TTD_FFI::Replay::ReplayEngine::Load(const u16* trace) const
{
    const std::filesystem::path tracePath {(const wchar_t*)trace};
    if ( !std::filesystem::exists(tracePath) )
    {
        err(L"File %S doesn't exist", tracePath.string().c_str());
        return ERROR_NOT_FOUND;
    }

    dbg(L"Loading trace %s", tracePath.wstring().c_str());
    if ( !this->m_Engine->Initialize(tracePath.wstring().c_str()) )
    {
        err(L"Initialize('%S') failed", tracePath.string().c_str());
        return LIBTTD_ERROR_INITIALIZATION;
    }

    ok(L"Loaded %s...", tracePath.wstring().c_str());
    return 0;
}

TTD::SystemInfo const&
TTD_FFI::Replay::ReplayEngine::GetSystemInfo() const
{
    return this->m_Engine->GetSystemInfo();
}


TTD::Replay::PositionRange const&
TTD_FFI::Replay::ReplayEngine::GetLifetime() const
{
    return this->m_Engine->GetLifetime();
}


u32
TTD_FFI::Replay::ReplayEngine::BuildIndex() const
{
    auto progress_cb = [](void const* pCallerContext, TTD::Replay::IndexBuildProgressType const* pProgressData) {};

    return std::underlying_type_t<TTD::Replay::IndexStatus>(
        this->m_Engine->BuildIndex(progress_cb, nullptr, TTD::Replay::IndexBuildFlags::None));
}

size_t
TTD_FFI::Replay::ReplayEngine::GetModuleCount() const
{

    return this->m_Engine->GetModuleCount();
}

TTD::Replay::Module const*
TTD_FFI::Replay::ReplayEngine::GetModuleList() const
{

    return this->m_Engine->GetModuleList();
}

size_t
TTD_FFI::Replay::ReplayEngine::GetModuleInstanceCount() const
{

    return this->m_Engine->GetModuleInstanceCount();
}

TTD::Replay::ModuleInstance const*
TTD_FFI::Replay::ReplayEngine::GetModuleInstanceList() const
{

    return this->m_Engine->GetModuleInstanceList();
}

size_t
TTD_FFI::Replay::ReplayEngine::GetThreadCount() const
{

    return this->m_Engine->GetThreadCount();
}

TTD::Replay::ThreadInfo const*
TTD_FFI::Replay::ReplayEngine::GetThreadList() const
{

    return this->m_Engine->GetThreadList();
}

size_t
TTD_FFI::Replay::ReplayEngine::GetModuleLoadedEventCount() const
{

    return this->m_Engine->GetModuleLoadedEventCount();
}

TTD::Replay::ModuleLoadedEvent const*
TTD_FFI::Replay::ReplayEngine::GetModuleLoadedEventList() const
{

    return this->m_Engine->GetModuleLoadedEventList();
}

size_t
TTD_FFI::Replay::ReplayEngine::GetModuleUnloadedEventCount() const
{

    return this->m_Engine->GetModuleUnloadedEventCount();
}

TTD::Replay::ModuleUnloadedEvent const*
TTD_FFI::Replay::ReplayEngine::GetModuleUnloadedEventList() const
{

    return this->m_Engine->GetModuleUnloadedEventList();
}

size_t
TTD_FFI::Replay::ReplayEngine::GetExceptionEventCount() const
{

    return this->m_Engine->GetExceptionEventCount();
}

TTD::Replay::ExceptionEvent const*
TTD_FFI::Replay::ReplayEngine::GetExceptionEventList() const
{
    return this->m_Engine->GetExceptionEventList();
}

#pragma endregion TTD_FFI::Replay::ReplayEngine

#pragma region TTD_FFI::Replay::ReplayCursor

TTD_FFI::Replay::ReplayCursor::ReplayCursor(const uptr raw_cursor) :
    m_Cursor {TTD::Replay::UniqueCursor((TTD::Replay::ICursor*)raw_cursor)}
{
    dbg(L"Allocating cursor");
}

TTD_FFI::Replay::ReplayCursor::~ReplayCursor()
{
    dbg(L"Destroying cursor");
    this->m_Cursor = nullptr;
}

i32
TTD_FFI::Replay::ReplayCursor::ReplayForward(
    TTD::Replay::Position const& limit,
    TTD::Replay::ICursorView::ReplayResult* out)
{
#ifdef _DEBUG
    std::array<wchar_t, 64> from {};
    std::array<wchar_t, 64> to {};

    const auto& cur = this->m_Cursor->GetPosition();
    TTD::Replay::PositionToString(cur, from.data(), from.size());
    TTD::Replay::PositionToString(limit, to.data(), from.size());
    dbg(L"Forward replaying from %s to %s", from.data(), to.data());
#endif // _DEBUG

    TTD::Replay::ICursorView::ReplayResult const res = this->m_Cursor->ReplayForward(limit);
    if ( res.StopReason == TTD::Replay::EventType::Invalid )
        return -1;

    ::memcpy(out, &res, sizeof(res));
    return 0;
}


i32
TTD_FFI::Replay::ReplayCursor::ReplayBackward(
    TTD::Replay::Position const& limit,
    TTD::Replay::ICursorView::ReplayResult* out)
{
#ifdef _DEBUG
    std::array<wchar_t, 64> from {};
    std::array<wchar_t, 64> to {};

    const auto& cur = this->m_Cursor->GetPosition();
    TTD::Replay::PositionToString(cur, from.data(), from.size());
    TTD::Replay::PositionToString(limit, to.data(), from.size());
    dbg(L"Backward replaying from %s to %s", from.data(), to.data());
#endif // _DEBUG

    TTD::Replay::ICursorView::ReplayResult const res = this->m_Cursor->ReplayBackward(limit);
    if ( res.StopReason == TTD::Replay::EventType::Invalid )
        return -1;

    ::memcpy(out, &res, sizeof(res));
    return 0;
}

i32
TTD_FFI::Replay::ReplayCursor::QueryMemoryBuffer(u64 address, u8* buf, usize bufsz) const
{

    dbg(L"QueryMemoryBuffer(addr=%llx ,buf=%p, bufsz=%llu)", address, buf, bufsz);
    const TTD::Replay::MemoryBuffer res = this->m_Cursor->QueryMemoryBuffer(
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
    dbg(L"Setting position %llx:%llx", (uint64_t)pos.Sequence, (uint64_t)pos.Steps);
    return this->m_Cursor->SetPosition(pos);
}

TTD::Replay::Position const&
TTD_FFI::Replay::ReplayCursor::GetPosition() const
{
    auto const& CurPos = this->m_Cursor->GetPosition();
    dbg(L"Current position is %llx:%llx", (uint64_t)CurPos.Sequence, (uint64_t)CurPos.Steps);
    return CurPos;
}

TTD::Replay::Position const&
TTD_FFI::Replay::ReplayCursor::GetPreviousPosition() const
{
    return this->m_Cursor->GetPreviousPosition();
}

TTD::Replay::ThreadInfo const&
TTD_FFI::Replay::ReplayCursor::GetThreadInfo() const
{
    return this->m_Cursor->GetThreadInfo();
}

u64
TTD_FFI::Replay::ReplayCursor::GetTebAddress() const
{
    return (u64)this->m_Cursor->GetTebAddress();
}

u64
TTD_FFI::Replay::ReplayCursor::GetProgramCounter() const
{
    return (u64)this->m_Cursor->GetProgramCounter();
}

u64
TTD_FFI::Replay::ReplayCursor::GetStackPointer() const
{
    return (u64)this->m_Cursor->GetStackPointer();
}

u64
TTD_FFI::Replay::ReplayCursor::GetFramePointer() const
{
    return (u64)this->m_Cursor->GetFramePointer();
}

X86_NT5_CONTEXT*
TTD_FFI::Replay::ReplayCursor::GetX86RegisterContext() const
{
    return reinterpret_cast<X86_NT5_CONTEXT*>(this->m_Cursor->GetCrossPlatformContext().Data);
}

AVX_EXTENDED_CONTEXT*
TTD_FFI::Replay::ReplayCursor::GetX86ExtendedRegisterContext() const
{
    return reinterpret_cast<AVX_EXTENDED_CONTEXT*>(this->m_Cursor->GetCrossPlatformContext().Data);
}

AMD64_CONTEXT*
TTD_FFI::Replay::ReplayCursor::GetX64RegisterContext() const
{
    return reinterpret_cast<AMD64_CONTEXT*>(this->m_Cursor->GetCrossPlatformContext().Data);
}

AVX_EXTENDED_CONTEXT*
TTD_FFI::Replay::ReplayCursor::GetX64ExtendedRegisterContext() const
{
    return reinterpret_cast<AVX_EXTENDED_CONTEXT*>(this->m_Cursor->GetCrossPlatformContext().Data);
}

ARM64_CONTEXT*
TTD_FFI::Replay::ReplayCursor::GetArm64RegisterContext() const
{
    return reinterpret_cast<ARM64_CONTEXT*>(this->m_Cursor->GetCrossPlatformContext().Data);
}

ARM64_NEON128*
TTD_FFI::Replay::ReplayCursor::GetArm64ExtendedRegisterContext() const
{
    return reinterpret_cast<ARM64_NEON128*>(this->m_Cursor->GetCrossPlatformContext().Data);
}


void
TTD_FFI::Replay::ReplayCursor::SetReplayFlags(TTD::Replay::ReplayFlags flags)
{
    return this->m_Cursor->SetReplayFlags(flags);
}

TTD::Replay::ReplayFlags
TTD_FFI::Replay::ReplayCursor::GetReplayFlags() const
{
    return this->m_Cursor->GetReplayFlags();
}

bool
TTD_FFI::Replay::ReplayCursor::AddMemoryWatchpoint(_In_ TTD::Replay::MemoryWatchpointData const& WatchPoint)
{
    return this->m_Cursor->AddMemoryWatchpoint(WatchPoint);
}

bool
TTD_FFI::Replay::ReplayCursor::RemoveMemoryWatchpoint(_In_ TTD::Replay::MemoryWatchpointData const& WatchPoint)
{
    return this->m_Cursor->RemoveMemoryWatchpoint(WatchPoint);
}

bool
TTD_FFI::Replay::ReplayCursor::AddPositionWatchpoint(_In_ TTD::Replay::PositionWatchpointData const& WatchPoint)
{
    return this->m_Cursor->AddPositionWatchpoint(WatchPoint);
}

bool
TTD_FFI::Replay::ReplayCursor::RemovePositionWatchpoint(_In_ TTD::Replay::PositionWatchpointData const& WatchPoint)
{
    return this->m_Cursor->RemovePositionWatchpoint(WatchPoint);
}

void
TTD_FFI::Replay::ReplayCursor::SetReplayProgressCallback(
    TTD::Replay::ICursorView::ReplayProgressCallback* cb,
    uptr context)
{
    this->m_Cursor->SetReplayProgressCallback(cb, context);
}

void
TTD_FFI::Replay::ReplayCursor::SetRegisterChangedCallback(
    TTD::Replay::ICursorView::RegisterChangedCallback* cb,
    uptr context)
{
    this->m_Cursor->SetRegisterChangedCallback(cb, context);
}

#pragma endregion TTD_FFI::Replay::ReplayCursor
