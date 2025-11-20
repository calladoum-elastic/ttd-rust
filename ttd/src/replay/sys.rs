//! Contains the thin wrapper for the unsafe stuff
use std::ffi::c_void;
use std::ops::BitOr;

use derive_more::Display;

use crate::bindings::root as ffi;
use crate::bindings::root::TTD::Replay::{IThreadView, Position};
use crate::prelude::*;
use crate::replay::ReplayPosition;

pub struct ReplayEngine {
    inner: ffi::TTD_FFI::Replay::ReplayEngine,
}

impl Drop for ReplayEngine {
    fn drop(&mut self) {
        unsafe {
            self.inner.destruct();
        }
    }
}

impl ReplayEngine {
    pub(crate) fn new() -> Result<Self> {
        let engine = unsafe {
            let mut engine = ffi::TTD_FFI::Replay::ReplayEngine::new();
            let eng_idx = engine.Index();
            if !(0..=ffi::TTD_FFI::Replay::MAX_ENGINE as i32).contains(&eng_idx) {
                return Err(Error::InitializationError);
            }
            engine
        };

        Ok(Self { inner: engine })
    }

    pub(crate) fn index(&self) -> i32 {
        unsafe { self.inner.Index() }
    }

    pub(crate) fn load(&self, trace: &[u16]) -> i32 {
        unsafe { self.inner.Load(trace.as_ptr()) }
    }

    pub fn cursor(&'_ self) -> Result<ReplayCursor<'_>> {
        let mut cursor = unsafe {
            let mut cursor = ffi::TTD_FFI::Replay::ReplayCursor::new(self.index());
            let cur_idx = cursor.Index();
            let eng_idx = cursor.EngineIndex();
            if !(0..=ffi::TTD_FFI::Replay::MAX_ENGINE as i32).contains(&eng_idx) || !(0..=ffi::TTD_FFI::Replay::MAX_CURSOR as i32).contains(&cur_idx) {
                return Err(Error::InitializationError);
            }
            cursor
        };

        // New cursors always should point to the start of the trace
        unsafe {
            cursor.SetPosition(&ffi::TTD::Replay::Position_Min);
        }
        Ok(ReplayCursor { inner: cursor, engine: self })
    }

    pub(crate) fn system_info(&self) -> &ffi::TTD::SystemInfo {
        unsafe { std::mem::transmute(self.inner.GetSystemInfo()) }
    }

    pub(crate) fn build_index(&self) -> u32 {
        unsafe { self.inner.BuildIndex() }
    }

    pub(crate) fn get_module_count(&self) -> usize {
        unsafe { self.inner.GetModuleCount() }
    }

    pub(crate) fn get_module_list(&self) -> Vec<ffi::TTD::Replay::Module> {
        unsafe { core::slice::from_raw_parts(self.inner.GetModuleList(), self.get_module_count()).into() }
    }

    pub(crate) fn get_module_instance_count(&self) -> usize {
        unsafe { self.inner.GetModuleInstanceCount() }
    }

    pub(crate) fn get_module_instance_list(&self) -> Vec<ffi::TTD::Replay::ModuleInstance> {
        unsafe { core::slice::from_raw_parts(self.inner.GetModuleInstanceList(), self.get_module_instance_count()).into() }
    }

    pub(crate) fn get_thread_count(&self) -> usize {
        unsafe { self.inner.GetThreadCount() }
    }

    pub(crate) fn get_thread_list(&self) -> Vec<ffi::TTD::Replay::ThreadInfo> {
        unsafe {
            let data = self.inner.GetThreadList();
            let cnt = self.get_thread_count();
            core::slice::from_raw_parts(data, cnt).into()
        }
    }

    pub(crate) fn get_module_loaded_event_count(&self) -> usize {
        unsafe { self.inner.GetModuleLoadedEventCount() }
    }

    pub(crate) fn get_module_loaded_event_list(&self) -> Vec<ffi::TTD::Replay::ModuleLoadedEvent> {
        unsafe {
            let data = self.inner.GetModuleLoadedEventList();
            let cnt = self.get_module_loaded_event_count();
            core::slice::from_raw_parts(data, cnt).into()
        }
    }

    pub(crate) fn get_module_unloaded_event_count(&self) -> usize {
        unsafe { self.inner.GetModuleUnloadedEventCount() }
    }

    pub(crate) fn get_module_unloaded_event_list(&self) -> Vec<ffi::TTD::Replay::ModuleUnloadedEvent> {
        unsafe {
            let data = self.inner.GetModuleUnloadedEventList();
            let cnt = self.get_module_unloaded_event_count();
            core::slice::from_raw_parts(data, cnt).into()
        }
    }

    pub(crate) fn get_exception_event_count(&self) -> usize {
        unsafe { self.inner.GetExceptionEventCount() }
    }

    pub(crate) fn get_exception_event_list(&self) -> Vec<ffi::TTD::Replay::ExceptionEvent> {
        unsafe {
            let data = self.inner.GetExceptionEventList();
            let cnt = self.get_exception_event_count();
            core::slice::from_raw_parts(data, cnt).into()
        }
    }
}

pub(crate) struct ReplayCursor<'a> {
    inner: ffi::TTD_FFI::Replay::ReplayCursor,
    engine: &'a ReplayEngine,
}

impl<'a> Drop for ReplayCursor<'a> {
    fn drop(&mut self) {
        unsafe {
            self.inner.destruct();
        }
    }
}

impl<'a> ReplayCursor<'a> {
    pub(crate) fn index(&self) -> i32 {
        self.inner.m_Index
    }

    pub(crate) fn engine_index(&self) -> i32 {
        self.inner.m_EngineIndex
    }

    pub(crate) fn replay_forward(&mut self, until: Option<ffi::TTD::Replay::Position>) -> ffi::TTD::Replay::ICursorView_ReplayResult {
        unsafe {
            match until {
                Some(pos) => self.inner.ReplayForward(&pos),
                None => self.inner.ReplayForward(&ffi::TTD::Replay::Position_Max),
            }
        }
    }

    pub(crate) fn replay_backward(&mut self, until: Option<ffi::TTD::Replay::Position>) -> ffi::TTD::Replay::ICursorView_ReplayResult {
        unsafe {
            match until {
                Some(pos) => self.inner.ReplayBackward(pos),
                None => self.inner.ReplayBackward(ffi::TTD::Replay::Position_Min),
            }
        }
    }

    pub(crate) fn set_position(&mut self, pos: &ffi::TTD::Replay::Position) {
        unsafe { self.inner.SetPosition(pos) };
    }

    pub(crate) fn get_position(&self) -> ffi::TTD::Replay::Position {
        unsafe { self.inner.GetPosition() }
    }

    pub(crate) fn get_previous_position(&mut self) -> ffi::TTD::Replay::Position {
        unsafe { self.inner.GetPreviousPosition() }
    }

    pub(crate) fn get_thread_info(&self) -> &ffi::TTD::Replay::ThreadInfo {
        unsafe { std::mem::transmute(self.inner.GetThreadInfo()) }
    }

    pub(crate) fn get_teb_address(&self) -> u64 {
        unsafe { self.inner.GetTebAddress() }
    }

    pub(crate) fn get_program_counter(&self) -> u64 {
        unsafe { self.inner.GetProgramCounter() }
    }

    pub(crate) fn get_stack_pointer(&self) -> u64 {
        unsafe { self.inner.GetStackPointer() }
    }

    pub(crate) fn get_frame_pointer(&self) -> u64 {
        unsafe { self.inner.GetFramePointer() }
    }

    pub(crate) fn get_thread_context(&self) -> Result<RegisterContext<'_>> {
        unsafe {
            let arch: ProcessorArchitecture = self.engine.system_info().System.ProcessorArchitecture.try_into()?;
            Ok(match arch {
                ProcessorArchitecture::X64 => {
                    let _ptr: *mut ffi::AMD64_CONTEXT = self.inner.GetX64RegisterContext();
                    let _ref: &ffi::AMD64_CONTEXT = std::mem::transmute(_ptr);
                    RegisterContext::X64(_ref)
                }
                ProcessorArchitecture::X86 => {
                    let _ptr: *mut ffi::X86_NT5_CONTEXT = self.inner.GetX86RegisterContext();
                    let _ref: &ffi::X86_NT5_CONTEXT = std::mem::transmute(_ptr);
                    RegisterContext::X86(_ref)
                }
            })
        }
    }

    pub(crate) fn get_thread_extended_context(&self) -> Result<ExtendedRegisterContext> {
        unsafe {
            let arch: ProcessorArchitecture = self.engine.system_info().System.ProcessorArchitecture.try_into()?;
            match arch {
                ProcessorArchitecture::X64 => Ok(ExtendedRegisterContext::X64(*self.inner.GetX64ExtendedRegisterContext())),
                ProcessorArchitecture::X86 => Ok(ExtendedRegisterContext::X86(*self.inner.GetX86ExtendedRegisterContext())),
            }
        }
    }

    pub(crate) fn read_current_memory(&self, address: u64, size: usize) -> Result<Vec<u8>> {
        let mut buffer = vec![0; size];
        let res = unsafe { self.inner.QueryMemoryBuffer(address, buffer.as_mut_ptr(), buffer.len() as u64) };

        match res {
            0 => Ok(buffer),
            _ => Err(Error::ForeignFunctionError),
        }
    }

    pub(crate) fn get_replay_flags(&self) -> ReplayFlags {
        unsafe { self.inner.GetReplayFlags().into() }
    }

    pub(crate) fn set_replay_flags(&mut self, flags: ReplayFlags) {
        unsafe { self.inner.SetReplayFlags(flags.into()) }
    }

    pub(crate) fn add_memory_watchpoint(&mut self, watch_point: &ffi::TTD::Replay::MemoryWatchpointData) -> bool {
        unsafe { self.inner.AddMemoryWatchpoint(watch_point) }
    }

    pub(crate) fn remove_memory_watchpoint(&mut self, watch_point: &ffi::TTD::Replay::MemoryWatchpointData) -> bool {
        unsafe { self.inner.RemoveMemoryWatchpoint(watch_point) }
    }

    pub(crate) fn add_position_watchpoint(&mut self, watch_point: &ffi::TTD::Replay::PositionWatchpointData) -> bool {
        unsafe { self.inner.AddPositionWatchpoint(watch_point) }
    }

    pub(crate) fn remove_position_watchpoint(&mut self, watch_point: &ffi::TTD::Replay::PositionWatchpointData) -> bool {
        unsafe { self.inner.RemovePositionWatchpoint(watch_point) }
    }

    pub(crate) fn set_register_changed_callback(&mut self, cb: RegisterChangedCallbackUnsafe) {
        unsafe { self.inner.SetRegisterChangedCallback(Some(cb), 0) }
    }
    pub(crate) fn set_replay_progress_callback(&mut self, cb: ReplayProgressCallbackUnsafe) {
        unsafe { self.inner.SetReplayProgressCallback(Some(cb), 0) }
    }
}

pub(crate) type RegisterChangedCallbackUnsafe =
    unsafe extern "C" fn(context: usize, reg_id: u8, old_data: *const c_void, new_data: *const c_void, data_size_in_bytes: usize, thread: *const IThreadView);

pub(crate) type ReplayProgressCallbackUnsafe = unsafe extern "C" fn(ctx: usize, pos: *const ffi::TTD::Replay::Position);

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

#[allow(clippy::large_enum_variant)]
pub enum RegisterContext<'a> {
    X64(&'a ffi::AMD64_CONTEXT),
    X86(&'a ffi::X86_NT5_CONTEXT),
}

impl<'a> std::fmt::Display for RegisterContext<'a> {
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

pub enum ExtendedRegisterContext {
    X64(ffi::AVX_EXTENDED_CONTEXT),
    X86(ffi::AVX_EXTENDED_CONTEXT),
}

#[cfg(test)]
mod test {
    use crate::{
        bindings::root::TTD::Replay::{Position, Position_Min},
        replay::sys::ReplayEngine,
    };

    fn get_test_trace() -> Vec<u16> {
        let mut trace_path = std::path::PathBuf::from(std::env::var("TEMP").expect("failed to get TEMP env var").as_str());
        trace_path.push("test.run");
        trace_path.to_string_lossy().encode_utf16().collect()
    }

    #[test]
    fn test_ffi_load_simple() {
        let replay = ReplayEngine::new().expect("failed to create a new replayer");
        assert!(replay.inner.m_Index >= 0);

        let trace_path = get_test_trace();
        assert_eq!(replay.load(trace_path.as_ref()), 0);

        for i in 1..10 {
            let mut cursor = replay.cursor().unwrap();
            let cursor_idx = cursor.index();
            assert_eq!(cursor_idx, 0);

            let pos = cursor.get_position();
            unsafe {
                assert_eq!(pos.Sequence, Position_Min.Sequence);
                assert_eq!(pos.Steps, Position_Min.Steps);
            }

            let res = cursor.replay_forward(None);
            assert_ne!(res.InstructionsExecuted, 0);
            assert_ne!(res.StopReason, 0);

            // cursor.replay_backward(None);
            assert_eq!(cursor_idx, cursor.index());
        }
    }

    #[test]
    fn sys_ffi_system_info() {
        let engine = ReplayEngine::new().expect("failed to create a new replayer");
        assert_eq!(engine.index(), 0);

        // Note: a trace is needed to have TTD::Replay::SystemInfo populated
        let trace_path = get_test_trace();
        assert_eq!(engine.load(trace_path.as_ref()), 0);

        let info = engine.system_info();
        assert_eq!(info.MajorVersion, 1);
        assert_eq!(info.MinorVersion, 9);
        assert_eq!(info.ProcessId, 11968);
        assert_eq!(info.SystemName.len(), 64);
        assert_eq!(info.UserName.len(), 64);
    }
}
