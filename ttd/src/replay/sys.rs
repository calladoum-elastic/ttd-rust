//! Contains the thin wrapper for the unsafe stuff
use std::ffi::c_void;
use std::ops::BitOr;

use derive_more::Display;

use crate::bindings::root as ffi;
use crate::bindings::root::TTD::Replay::IThreadView;
use crate::prelude::*;

pub(crate) fn initialize() -> i32 {
    unsafe { ffi::TTD_FFI::Replay::Initialize() }
}

pub(crate) fn reset() -> i32 {
    unsafe { ffi::TTD_FFI::Replay::Reset() }
}

pub(crate) fn load(trace: &str) -> i32 {
    unsafe { ffi::TTD_FFI::Replay::Load(trace.as_ptr()) }
}

pub(crate) fn system_info() -> ffi::TTD::SystemInfo {
    unsafe { ffi::TTD_FFI::Replay::GetSystemInfo() }
}

pub(crate) fn build_index() -> u32 {
    unsafe { ffi::TTD_FFI::Replay::BuildIndex() }
}

pub(crate) fn replay_forward(until: Option<ffi::TTD::Replay::Position>) -> ffi::TTD::Replay::ICursorView_ReplayResult {
    unsafe {
        match until {
            Some(pos) => ffi::TTD_FFI::Replay::ReplayForward1(pos),
            None => ffi::TTD_FFI::Replay::ReplayForward(),
        }
    }
}

pub(crate) fn replay_backward(until: Option<ffi::TTD::Replay::Position>) -> ffi::TTD::Replay::ICursorView_ReplayResult {
    unsafe {
        match until {
            Some(pos) => ffi::TTD_FFI::Replay::ReplayBackward1(pos),
            None => ffi::TTD_FFI::Replay::ReplayBackward(),
        }
    }
}

pub(crate) type RegisterChangedCallbackUnsafe =
    unsafe extern "C" fn(context: usize, reg_id: u8, old_data: *const c_void, new_data: *const c_void, data_size_in_bytes: usize, thread: *const IThreadView);

pub(crate) fn set_register_changed_callback(cb: RegisterChangedCallbackUnsafe) {
    unsafe { ffi::TTD_FFI::Replay::SetRegisterChangedCallback(Some(cb), 0) }
}

pub(crate) type ReplayProgressCallbackUnsafe = unsafe extern "C" fn(ctx: usize, pos: *const ffi::TTD::Replay::Position);

pub(crate) fn set_replay_progress_callback(cb: ReplayProgressCallbackUnsafe) {
    unsafe { ffi::TTD_FFI::Replay::SetReplayProgressCallback(Some(cb), 0) }
}

pub(crate) fn set_position(pos: &ffi::TTD::Replay::Position) {
    unsafe { ffi::TTD_FFI::Replay::SetPosition(pos) };
}

pub(crate) fn get_position() -> ffi::TTD::Replay::Position {
    unsafe { ffi::TTD_FFI::Replay::GetPosition() }
}

pub(crate) fn read_current_memory(address: u64, size: usize) -> Result<Vec<u8>> {
    let mut buffer = vec![0; size];
    let res = unsafe { ffi::TTD_FFI::Replay::QueryMemoryBuffer(address, buffer.as_mut_ptr(), buffer.len() as u64) };

    match res {
        0 => Ok(buffer),
        _ => Err(Error::ForeignFunctionError),
    }
}

pub(crate) fn get_thread_info() -> ffi::TTD::Replay::ThreadInfo {
    unsafe { *ffi::TTD_FFI::Replay::GetThreadInfo() }
}

pub(crate) fn get_previous_position() -> ffi::TTD::Replay::Position {
    unsafe { ffi::TTD_FFI::Replay::GetPreviousPosition() }
}

pub(crate) fn get_teb_address() -> u64 {
    unsafe { ffi::TTD_FFI::Replay::GetTebAddress() }
}

pub(crate) fn get_program_counter() -> u64 {
    unsafe { ffi::TTD_FFI::Replay::GetProgramCounter() }
}

pub(crate) fn get_stack_pointer() -> u64 {
    unsafe { ffi::TTD_FFI::Replay::GetStackPointer() }
}

pub(crate) fn get_frame_pointer() -> u64 {
    unsafe { ffi::TTD_FFI::Replay::GetFramePointer() }
}

#[repr(u32)]
#[derive(Default, Display)]
pub enum ReplayFlags {
    #[default]
    None = 0,
    ReplayOnlyCurrentThread = 0x00000001,
    ReplayAllSegmentsWithoutFiltering = 0x00000002,
    ReplaySegmentsSequentially = 0x00000004,
}
impl From<ReplayFlags> for u32 {
    fn from(val: ReplayFlags) -> Self {
        val as u32
    }
}

impl From<u32> for ReplayFlags {
    fn from(value: u32) -> Self {
        match value {
            0 => Self::default(),
            1 => ReplayFlags::ReplayOnlyCurrentThread,
            2 => ReplayFlags::ReplayAllSegmentsWithoutFiltering,
            4 => ReplayFlags::ReplaySegmentsSequentially,
            _ => unimplemented!(),
        }
    }
}

pub(crate) fn get_replay_flags() -> ReplayFlags {
    unsafe { ffi::TTD_FFI::Replay::GetReplayFlags().into() }
}

pub(crate) fn set_replay_flags(flags: ReplayFlags) {
    unsafe { ffi::TTD_FFI::Replay::SetReplayFlags(flags.into()) }
}

#[allow(clippy::large_enum_variant)]
pub enum RegisterContext {
    X64(ffi::AMD64_CONTEXT),
    X86(ffi::X86_NT5_CONTEXT),
}

impl std::fmt::Display for RegisterContext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RegisterContext::X64(ctx) => {
                write!(
                    f,
                    "rax={:016x} rbx={:016x} rcx={:016x}
rdx={:016x} rsi={:016x} rdi={:016x}
rip={:016x} rsp={:016x} rbp={:016x}
 r8={:016x}  r9={:016x} r10={:016x}
r11={:016x} r12={:016x} r13={:016x}
r14={:016x} r15={:016x}",
                    ctx.Rax,
                    ctx.Rbx,
                    ctx.Rcx,
                    ctx.Rdx,
                    ctx.Rsi,
                    ctx.Rdi,
                    ctx.Rip,
                    ctx.Rsp,
                    ctx.Rbp,
                    ctx.R8,
                    ctx.R9,
                    ctx.R10,
                    ctx.R11,
                    ctx.R12,
                    ctx.R13,
                    ctx.R14,
                    ctx.R15
                )
            }
            RegisterContext::X86(ctx) => write!(
                f,
                "eax={:08x} ebx={:08x} ecx={:08x} edx={:08x} esi={:08x} edi={:08x}
eip={:08x} esp={:08x} ebp={:08x} ",
                ctx.Eax, ctx.Ebx, ctx.Ecx, ctx.Edx, ctx.Esi, ctx.Edi, ctx.Eip, ctx.Esp, ctx.Ebp,
            ),
        }
    }
}

#[repr(u16)]
pub(crate) enum ProcessorArchitecture {
    X64 = 9,
    X86 = 0,
    // ARM64 = 12,
}
impl TryFrom<u16> for ProcessorArchitecture {
    type Error = crate::error::Error;

    fn try_from(value: u16) -> Result<Self> {
        match value {
            9 => Ok(ProcessorArchitecture::X64),
            0 => Ok(ProcessorArchitecture::X86),
            _ => Err(Error::ConversionError),
        }
    }
}

pub(crate) fn get_thread_context() -> Result<RegisterContext> {
    unsafe {
        let arch: ProcessorArchitecture = system_info().System.ProcessorArchitecture.try_into()?;
        match arch {
            ProcessorArchitecture::X64 => Ok(RegisterContext::X64(ffi::TTD_FFI::Replay::GetX64RegisterContext())),
            ProcessorArchitecture::X86 => Ok(RegisterContext::X86(ffi::TTD_FFI::Replay::GetX86RegisterContext())),
        }
    }
}

pub enum ExtendedRegisterContext {
    X64(ffi::AVX_EXTENDED_CONTEXT),
    X86(ffi::AVX_EXTENDED_CONTEXT),
}

pub(crate) fn get_thread_extended_context() -> Result<ExtendedRegisterContext> {
    unsafe {
        let arch: ProcessorArchitecture = system_info().System.ProcessorArchitecture.try_into()?;
        match arch {
            ProcessorArchitecture::X64 => Ok(ExtendedRegisterContext::X64(ffi::TTD_FFI::Replay::GetX64ExtendedRegisterContext())),
            ProcessorArchitecture::X86 => Ok(ExtendedRegisterContext::X86(ffi::TTD_FFI::Replay::GetX86ExtendedRegisterContext())),
        }
    }
}

pub(crate) fn add_memory_watchpoint(watch_point: &ffi::TTD::Replay::MemoryWatchpointData) -> bool {
    unsafe { ffi::TTD_FFI::Replay::AddMemoryWatchpoint(watch_point) }
}

pub(crate) fn remove_memory_watchpoint(watch_point: &ffi::TTD::Replay::MemoryWatchpointData) -> bool {
    unsafe { ffi::TTD_FFI::Replay::RemoveMemoryWatchpoint(watch_point) }
}

pub(crate) fn add_position_watchpoint(watch_point: &ffi::TTD::Replay::PositionWatchpointData) -> bool {
    unsafe { ffi::TTD_FFI::Replay::AddPositionWatchpoint(watch_point) }
}

pub(crate) fn remove_position_watchpoint(watch_point: &ffi::TTD::Replay::PositionWatchpointData) -> bool {
    unsafe { ffi::TTD_FFI::Replay::RemovePositionWatchpoint(watch_point) }
}

pub(crate) fn get_module_count() -> usize {
    unsafe { ffi::TTD_FFI::Replay::GetModuleCount() }
}

pub(crate) fn get_module_list() -> Vec<ffi::TTD::Replay::Module> {
    unsafe { core::slice::from_raw_parts(ffi::TTD_FFI::Replay::GetModuleList(), get_module_count()).into() }
}

pub(crate) fn get_module_instance_count() -> usize {
    unsafe { ffi::TTD_FFI::Replay::GetModuleInstanceCount() }
}

pub(crate) fn get_module_instance_list() -> Vec<ffi::TTD::Replay::ModuleInstance> {
    unsafe { core::slice::from_raw_parts(ffi::TTD_FFI::Replay::GetModuleInstanceList(), get_module_instance_count()).into() }
}

pub(crate) fn get_thread_count() -> usize {
    unsafe { ffi::TTD_FFI::Replay::GetThreadCount() }
}

pub(crate) fn get_thread_list() -> Vec<ffi::TTD::Replay::ThreadInfo> {
    unsafe {
        let data = ffi::TTD_FFI::Replay::GetThreadList();
        let cnt = get_thread_count();
        core::slice::from_raw_parts(data, cnt).into()
    }
}

pub(crate) fn get_module_loaded_event_count() -> usize {
    unsafe { ffi::TTD_FFI::Replay::GetModuleLoadedEventCount() }
}

pub(crate) fn get_module_loaded_event_list() -> Vec<ffi::TTD::Replay::ModuleLoadedEvent> {
    unsafe {
        let data = ffi::TTD_FFI::Replay::GetModuleLoadedEventList();
        let cnt = get_module_loaded_event_count();
        core::slice::from_raw_parts(data, cnt).into()
    }
}

pub(crate) fn get_module_unloaded_event_count() -> usize {
    unsafe { ffi::TTD_FFI::Replay::GetModuleUnloadedEventCount() }
}

pub(crate) fn get_module_unloaded_event_list() -> Vec<ffi::TTD::Replay::ModuleUnloadedEvent> {
    unsafe {
        let data = ffi::TTD_FFI::Replay::GetModuleUnloadedEventList();
        let cnt = get_module_unloaded_event_count();
        core::slice::from_raw_parts(data, cnt).into()
    }
}

pub(crate) fn get_exception_event_count() -> usize {
    unsafe { ffi::TTD_FFI::Replay::GetExceptionEventCount() }
}

pub(crate) fn get_exception_event_list() -> Vec<ffi::TTD::Replay::ExceptionEvent> {
    unsafe {
        let data = ffi::TTD_FFI::Replay::GetExceptionEventList();
        let cnt = get_exception_event_count();
        core::slice::from_raw_parts(data, cnt).into()
    }
}
