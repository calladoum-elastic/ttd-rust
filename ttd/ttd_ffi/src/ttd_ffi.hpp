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

#include "TTD/IReplayEngine.h"
#include "TTD/IReplayEngineRegisters.h"
#include "TTD/IReplayEngineStl.h"

namespace TTD_FFI::Replay
{

/// Initialize the engine
i32
Initialize();

/// Load trace and allocate cursor
i32
Load(const u16* trace);

/// Unload the engine
i32
Reset();

/// Get system info from the replay engine
TTD::SystemInfo
GetSystemInfo();

/// Build a trace index, to boost search speed
u32
BuildIndex();

TTD::Replay::ICursorView::ReplayResult
ReplayForward();

TTD::Replay::ICursorView::ReplayResult
ReplayForward(TTD::Replay::Position limit);

TTD::Replay::ICursorView::ReplayResult
ReplayBackward();

TTD::Replay::ICursorView::ReplayResult
ReplayBackward(TTD::Replay::Position limit);

void
SetPosition(TTD::Replay::Position const& pos);

TTD::Replay::Position
GetPosition();

i32
QueryMemoryBuffer(u64 address, u8* buf, usize bufsz);

TTD::Replay::ThreadInfo const&
GetThreadInfo();

TTD::Replay::Position
GetPreviousPosition();

u64
GetTebAddress();

u64
GetProgramCounter();

u64
GetStackPointer();

u64
GetFramePointer();

X86_NT5_CONTEXT
GetX86RegisterContext();

AVX_EXTENDED_CONTEXT
GetX86ExtendedRegisterContext();

AMD64_CONTEXT
GetX64RegisterContext();

AVX_EXTENDED_CONTEXT
GetX64ExtendedRegisterContext();

void
SetReplayFlags(TTD::Replay::ReplayFlags flags);

TTD::Replay::ReplayFlags
GetReplayFlags();

bool
AddMemoryWatchpoint(TTD::Replay::MemoryWatchpointData const&);

bool
RemoveMemoryWatchpoint(TTD::Replay::MemoryWatchpointData const&);

bool
AddPositionWatchpoint(TTD::Replay::PositionWatchpointData const&);

bool
RemovePositionWatchpoint(TTD::Replay::PositionWatchpointData const&);

size_t
GetModuleCount();

TTD::Replay::Module const*
GetModuleList();

size_t
GetModuleInstanceCount();

TTD::Replay::ModuleInstance const*
GetModuleInstanceList();

size_t
GetThreadCount();

TTD::Replay::ThreadInfo const*
GetThreadList();

size_t
GetModuleLoadedEventCount();

TTD::Replay::ModuleLoadedEvent const*
GetModuleLoadedEventList();

size_t
GetModuleUnloadedEventCount();

TTD::Replay::ModuleUnloadedEvent const*
GetModuleUnloadedEventList();

size_t
GetExceptionEventCount();

TTD::Replay::ExceptionEvent const*
GetExceptionEventList();

// Cursor Callbacks

void
SetReplayProgressCallback(TTD::Replay::ICursorView::ReplayProgressCallback* cb, uptr context);

void
SetRegisterChangedCallback(TTD::Replay::ICursorView::RegisterChangedCallback*, uptr context);

// End of Cursor Callbacks

} // namespace TTD_FFI::Replay
#endif // !__HAS_TTD_FFI
