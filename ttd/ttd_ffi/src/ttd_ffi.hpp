#pragma once

#ifndef __HAS_TTD_FFI
#define __HAS_TTD_FFI

#include "constants.hpp"

using i8    = char;
using u8    = unsigned char;
using i16   = short;
using u16   = unsigned short;
using i32   = int;
using u32   = unsigned int;
using i64   = long long;
using u64   = unsigned long long;
using isize = i64;
using usize = u64;
using uptr  = usize;

#include <vector>

#include "TTD/IReplayEngine.h"
#include "TTD/IReplayEngineRegisters.h"
#include "TTD/IReplayEngineStl.h"


#define LIBTTD_INVALID_VALUE ((i32) - 1)
#define LIBTTD_ERROR_GENERIC ((i32) - 1)
#define LIBTTD_ERROR_NOT_FOUND ((i32) - 2)
#define LIBTTD_ERROR_INITIALIZATION ((i32) - 3)
#define LIBTTD_ERROR_INVALID_INDEX ((i32) - 4)


namespace TTD_FFI::Replay
{

const static size_t MAX_ENGINE = 256;
const static size_t MAX_CURSOR = 256;


class ReplayEngine
{
private:
    i32 m_Index {LIBTTD_INVALID_VALUE};

    /// Initialize the engine
    i32
    Initialize();

public:
    ReplayEngine();

    ~ReplayEngine();

    /// Get the index of the allocated engine
    i32
    Index() const;


    /// Load trace from the `.run` filepath passed as argument
    i32
    Load(const u16* trace) const;

    /// Unload the engine
    i32
    Reset();

    /// Get the trace entire life range
    TTD::Replay::PositionRange const&
    GetLifetime() const;

    /// Get system info from the replay engine
    TTD::SystemInfo const&
    GetSystemInfo() const;

    /// Build a trace index, to boost search speed
    u32
    BuildIndex() const;

    size_t
    GetModuleCount() const;

    TTD::Replay::Module const*
    GetModuleList() const;

    size_t
    GetModuleInstanceCount() const;

    TTD::Replay::ModuleInstance const*
    GetModuleInstanceList() const;

    size_t
    GetThreadCount() const;

    TTD::Replay::ThreadInfo const*
    GetThreadList() const;

    size_t
    GetModuleLoadedEventCount() const;

    TTD::Replay::ModuleLoadedEvent const*
    GetModuleLoadedEventList() const;

    size_t
    GetModuleUnloadedEventCount() const;

    TTD::Replay::ModuleUnloadedEvent const*
    GetModuleUnloadedEventList() const;

    size_t
    GetExceptionEventCount() const;

    TTD::Replay::ExceptionEvent const*
    GetExceptionEventList() const;
};


class ReplayCursor
{
private:
    i32 m_Index;
    i32 m_EngineIndex;

    /// Initialize the cursor with a specific engine
    i32 Initialize(i32);

public:
    ReplayCursor(i32);

    ~ReplayCursor();

    i32
    Index() const;

    i32
    EngineIndex() const;

    /// Unload the cursor
    i32
    Reset();

    TTD::Replay::ICursorView::ReplayResult
    ReplayForward(TTD::Replay::Position const& limit);

    TTD::Replay::ICursorView::ReplayResult
    ReplayBackward(TTD::Replay::Position const& limit);

    void
    SetPosition(TTD::Replay::Position const& pos);

    TTD::Replay::Position const&
    GetPosition() const;

    TTD::Replay::Position const&
    GetPreviousPosition() const;

    TTD::Replay::ThreadInfo const&
    GetThreadInfo() const;

    u64
    GetTebAddress() const;

    u64
    GetProgramCounter() const;

    u64
    GetStackPointer() const;

    u64
    GetFramePointer() const;

    X86_NT5_CONTEXT*
    GetX86RegisterContext() const;

    AVX_EXTENDED_CONTEXT*
    GetX86ExtendedRegisterContext() const;

    AMD64_CONTEXT*
    GetX64RegisterContext() const;

    AVX_EXTENDED_CONTEXT*
    GetX64ExtendedRegisterContext() const;

    void
    SetReplayFlags(TTD::Replay::ReplayFlags flags);

    TTD::Replay::ReplayFlags
    GetReplayFlags() const;

    i32
    QueryMemoryBuffer(u64 address, u8* buf, usize bufsz) const;

    bool
    AddMemoryWatchpoint(TTD::Replay::MemoryWatchpointData const&);

    bool
    RemoveMemoryWatchpoint(TTD::Replay::MemoryWatchpointData const&);

    bool
    AddPositionWatchpoint(TTD::Replay::PositionWatchpointData const&);

    bool
    RemovePositionWatchpoint(TTD::Replay::PositionWatchpointData const&);


#pragma region Cursor Callbacks
    void
    SetReplayProgressCallback(TTD::Replay::ICursorView::ReplayProgressCallback* cb, uptr context);

    void
    SetRegisterChangedCallback(TTD::Replay::ICursorView::RegisterChangedCallback*, uptr context);
#pragma endregion Cursor Callbacks
};


} // namespace TTD_FFI::Replay
#endif // !__HAS_TTD_FFI
