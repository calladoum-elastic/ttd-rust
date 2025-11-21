//! Contains the replay module
// Most operations here are exposed as safe
//
use derive_more::Display;

use crate::{bindings::root::TTD::Replay::PositionRange, prelude::*};

pub(crate) mod sys;

use std::{
    ffi::CString,
    ops::{Add, Sub},
    os::windows::ffi::OsStringExt,
    str::FromStr,
};

use bitflags::bitflags;

pub type SystemInfo = crate::bindings::root::TTD::SystemInfo;
pub type ThreadInfo = crate::bindings::root::TTD::Replay::ThreadInfo;
pub type ReplayPosition = crate::bindings::root::TTD::Replay::Position;
pub type ReplayPositionRange = crate::bindings::root::TTD::Replay::PositionRange;
pub type ThreadView = crate::bindings::root::TTD::Replay::IThreadView;
pub type Amd64Context = crate::bindings::root::AMD64_CONTEXT;
pub type Amd64ExtendedContext = crate::bindings::root::AVX_EXTENDED_CONTEXT;

pub type ReplayFlags = crate::replay::sys::ReplayFlags;
pub type RegisterContext<'a> = crate::replay::sys::RegisterContext<'a>;
pub type ExtendedRegisterContext = crate::replay::sys::ExtendedRegisterContext;

pub type SequenceId = crate::bindings::root::TTD::SequenceId;
pub type PositionWatchpointData = crate::bindings::root::TTD::Replay::PositionWatchpointData;
pub type MemoryWatchpointData = crate::bindings::root::TTD::Replay::MemoryWatchpointData;

impl Add<u64> for ReplayPosition {
    type Output = ReplayPosition;

    fn add(mut self, rhs: u64) -> Self::Output {
        self.Steps += rhs;
        self
    }
}

impl Sub<u64> for ReplayPosition {
    type Output = ReplayPosition;

    fn sub(mut self, rhs: u64) -> Self::Output {
        self.Steps -= rhs;
        self
    }
}

impl std::fmt::Display for ReplayPosition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:x}:{:x}", self.Sequence, self.Steps)
    }
}

pub type ReplayProgressCallback = fn(ctx: usize, pos: &ReplayPosition);
pub type RegisterChangedCallback = fn(context: usize, reg_id: u8, old_data: &[u8; 8], new_data: &[u8; 8], ddata_size_in_bytes: usize, thread: &ThreadView);

#[derive(Display, Debug, PartialEq)]
pub enum EventType {
    MemoryWatchpoint,
    PositionWatchpoint,
    Exception,
    Gap,
    Thread,
    StepCount,
    Position,
    Process,
    Interrupted,
    Error,
    Count,
    Invalid,
}

impl From<u8> for EventType {
    fn from(value: u8) -> Self {
        match value {
            crate::bindings::root::TTD::Replay::EventType_MemoryWatchpoint => EventType::MemoryWatchpoint,
            crate::bindings::root::TTD::Replay::EventType_PositionWatchpoint => EventType::PositionWatchpoint,
            crate::bindings::root::TTD::Replay::EventType_Exception => EventType::Exception,
            crate::bindings::root::TTD::Replay::EventType_Gap => EventType::Gap,
            crate::bindings::root::TTD::Replay::EventType_Thread => EventType::Thread,
            crate::bindings::root::TTD::Replay::EventType_StepCount => EventType::StepCount,
            crate::bindings::root::TTD::Replay::EventType_Position => EventType::Position,
            crate::bindings::root::TTD::Replay::EventType_Process => EventType::Process,
            crate::bindings::root::TTD::Replay::EventType_Interrupted => EventType::Interrupted,
            crate::bindings::root::TTD::Replay::EventType_Error => EventType::Error,
            crate::bindings::root::TTD::Replay::EventType_Count => EventType::Count,
            _ => EventType::Invalid,
        }
    }
}

bitflags! {
    pub struct DataAccessType: u8 {
        const Read          = crate::bindings::root::TTD::Replay::DataAccessType_Read;
        const Write         =      crate::bindings::root::TTD::Replay::DataAccessType_Write;
        const Execute       =     crate::bindings::root::TTD::Replay::DataAccessType_Execute;
        const CodeFetch     =     crate::bindings::root::TTD::Replay::DataAccessType_CodeFetch;
        const Overwrite     =     crate::bindings::root::TTD::Replay::DataAccessType_Overwrite;
        const DataMismatch  =     crate::bindings::root::TTD::Replay::DataAccessType_DataMismatch;
        const NewData       =     crate::bindings::root::TTD::Replay::DataAccessType_NewData;
        const RedundantData =     crate::bindings::root::TTD::Replay::DataAccessType_RedundantData;
    }
}

bitflags! {
    pub struct DataAccessMask: u8 {
    const Read          = crate::bindings::root::TTD::Replay::DataAccessMask_Read;
    const Write         = crate::bindings::root::TTD::Replay::DataAccessMask_Write;
    const Execute       = crate::bindings::root::TTD::Replay::DataAccessMask_Execute;
    const CodeFetch     = crate::bindings::root::TTD::Replay::DataAccessMask_CodeFetch;
    const Overwrite     = crate::bindings::root::TTD::Replay::DataAccessMask_Overwrite;
    const DataMismatch  = crate::bindings::root::TTD::Replay::DataAccessMask_DataMismatch;
    const NewData       = crate::bindings::root::TTD::Replay::DataAccessMask_NewData;
    const RedundantData = crate::bindings::root::TTD::Replay::DataAccessMask_RedundantData;
    const None      = crate::bindings::root::TTD::Replay::DataAccessMask_None;
    const ReadWrite = crate::bindings::root::TTD::Replay::DataAccessMask_ReadWrite;
    const All       = crate::bindings::root::TTD::Replay::DataAccessMask_All;
}
}

#[derive(Debug)]
pub struct ReplayModule {
    pub name: String,
    pub address: u64,
    pub size: u64,
    pub checksum: u32,
    pub timestamp: u32,
}

impl TryFrom<&crate::bindings::root::TTD::Replay::Module> for ReplayModule {
    fn try_from(value: &crate::bindings::root::TTD::Replay::Module) -> Result<Self> {
        let name_slice = unsafe { std::slice::from_raw_parts(value.pName, value.NameLength) };

        Ok(Self {
            name: String::from_utf16(name_slice)?.to_string(),
            address: value.Address,
            size: value.Size,
            checksum: value.Checksum,
            timestamp: value.Timestamp,
        })
    }

    type Error = crate::error::Error;
}

#[derive(Debug)]
pub struct ModuleInstance {
    pub module: ReplayModule,
    pub load_time: SequenceId,
    pub unload_time: SequenceId,
}

#[derive(Debug)]
pub struct ActiveThreadInfo {
    pub thread: ThreadInfo,
    pub current_position: ReplayPosition,
    pub last_valid_position: ReplayPosition,
}

pub struct ReplayResult {
    pub stop_reason: EventType,
    pub steps_executed: u64,
    pub instructions_executed: u64,
}

impl From<crate::bindings::root::TTD::Replay::ICursorView_ReplayResult> for ReplayResult {
    fn from(value: crate::bindings::root::TTD::Replay::ICursorView_ReplayResult) -> Self {
        Self {
            stop_reason: value.StopReason.into(),
            steps_executed: value.StepsExecuted,
            instructions_executed: value.InstructionsExecuted,
        }
    }
}

pub mod events {

    use derive_more::Display;

    use crate::prelude::*;
    use crate::replay::{ReplayModule, ReplayPosition};

    #[derive(Debug)]
    pub struct ModuleLoaded {
        pub position: ReplayPosition,
        pub module: ReplayModule,
    }

    impl TryFrom<&crate::bindings::root::TTD::Replay::ModuleLoadedEvent> for ModuleLoaded {
        fn try_from(value: &crate::bindings::root::TTD::Replay::ModuleLoadedEvent) -> Result<Self> {
            let module = unsafe { (*value.pModule) };
            Ok(Self {
                position: value.Position,
                module: ReplayModule::try_from(&module)?,
            })
        }
        type Error = crate::error::Error;
    }

    #[derive(Debug)]
    pub struct ModuleUnloaded {
        pub position: ReplayPosition,
        pub module: ReplayModule,
    }
    impl TryFrom<&crate::bindings::root::TTD::Replay::ModuleUnloadedEvent> for ModuleUnloaded {
        fn try_from(value: &crate::bindings::root::TTD::Replay::ModuleUnloadedEvent) -> Result<Self> {
            let module = unsafe { (*value.pModule) };
            Ok(Self {
                position: value.Position,
                module: ReplayModule::try_from(&module)?,
            })
        }
        type Error = crate::error::Error;
    }

    #[derive(Debug)]
    pub struct Exception {}
    impl TryFrom<&crate::bindings::root::TTD::Replay::ExceptionEvent> for Exception {
        fn try_from(value: &crate::bindings::root::TTD::Replay::ExceptionEvent) -> Result<Self> {
            todo!()
        }
        type Error = crate::error::Error;
    }
}

// region: EngineInfo
pub struct EngineInfo {
    pub major: usize,
    pub minor: usize,
    pub patch: usize,
    pub license: String,
    pub author: String,
    pub banner: String,
    pub name: String,
}

impl Default for EngineInfo {
    fn default() -> Self {
        Self::new()
    }
}

impl EngineInfo {
    pub fn new() -> Self {
        let license = unsafe { CString::from_vec_unchecked(crate::bindings::root::TTD_FFI::LibraryLicense.to_vec()) };

        let author = unsafe { CString::from_vec_unchecked(crate::bindings::root::TTD_FFI::LibraryAuthor.to_vec()) };

        let banner = unsafe { CString::from_vec_unchecked(crate::bindings::root::TTD_FFI::LibraryBanner.to_vec()) };

        let name = unsafe { CString::from_vec_unchecked(crate::bindings::root::TTD_FFI::LibraryName.to_vec()) };

        EngineInfo {
            major: crate::bindings::root::TTD_FFI::LibraryVersionMajor,
            minor: crate::bindings::root::TTD_FFI::LibraryVersionMinor,
            patch: crate::bindings::root::TTD_FFI::LibraryVersionPatch,
            license: license.to_string_lossy().into(),
            author: author.to_string_lossy().into(),
            banner: banner.to_string_lossy().into(),
            name: name.to_string_lossy().into(),
        }
    }
}
// endregion: EngineInfo

pub struct ReplayCursor<'a> {
    inner: crate::replay::sys::ReplayCursor<'a>,
}

impl<'a> ReplayCursor<'a> {
    pub fn replay_forward(&mut self, until: Option<ReplayPosition>) -> Result<ReplayResult> {
        Ok(self.inner.replay_forward(until)?.into())
    }

    pub fn replay_backward(&mut self, until: Option<ReplayPosition>) -> Result<ReplayResult> {
        Ok(self.inner.replay_backward(until)?.into())
    }

    pub fn replay_forward_steps(&mut self, steps: u64) -> Result<ReplayResult> {
        let until = *self.inner.get_position() + steps;
        Ok(self.inner.replay_forward(Some(until))?.into())
    }

    pub fn replay_backward_steps(&mut self, steps: u64) -> Result<ReplayResult> {
        let until = *self.inner.get_position() + steps;
        Ok(self.inner.replay_backward(Some(until))?.into())
    }

    pub fn set_position(&mut self, pos: &ReplayPosition) {
        self.inner.set_position(pos)
    }

    pub fn get_position(&self) -> Result<&ReplayPosition> {
        Ok(self.inner.get_position())
    }

    pub fn get_previous_position(&mut self) -> Result<&ReplayPosition> {
        Ok(self.inner.get_previous_position())
    }

    pub fn get_thread_info(&self) -> Result<&ThreadInfo> {
        Ok(self.inner.get_thread_info())
    }

    pub fn get_teb_address(&self) -> Result<u64> {
        Ok(self.inner.get_teb_address())
    }

    pub fn get_program_counter(&self) -> Result<u64> {
        Ok(self.inner.get_program_counter())
    }

    pub fn get_stack_pointer(&self) -> Result<u64> {
        Ok(self.inner.get_stack_pointer())
    }

    pub fn get_frame_pointer(&self) -> Result<u64> {
        Ok(self.inner.get_frame_pointer())
    }

    pub fn get_thread_context(&self) -> Result<RegisterContext<'_>> {
        self.inner.get_thread_context()
    }

    pub fn pointer_size(&self) -> Result<usize> {
        match self.get_thread_context()? {
            sys::RegisterContext::X64(_) => Ok(8),
            sys::RegisterContext::X86(_) => Ok(4),
            sys::RegisterContext::ARM64(_) => Ok(8),
        }
    }

    pub fn get_thread_extended_context(&self) {
        unimplemented!()
    }

    pub fn read_current_memory(&self, address: u64, size: usize) -> Result<Vec<u8>> {
        self.inner.read_current_memory(address, size)
    }

    pub fn get_replay_flags(&self) -> Result<ReplayFlags> {
        Ok(self.inner.get_replay_flags())
    }

    pub fn set_replay_flags(&mut self, flags: ReplayFlags) -> Result<()> {
        self.inner.set_replay_flags(flags);
        Ok(())
    }

    pub fn add_memory_watchpoint(&mut self, watch_point: &MemoryWatchpointData) -> Result<bool> {
        Ok(self.inner.add_memory_watchpoint(watch_point))
    }

    pub fn remove_memory_watchpoint(&mut self, watch_point: &MemoryWatchpointData) -> Result<bool> {
        Ok(self.inner.remove_memory_watchpoint(watch_point))
    }

    pub fn add_position_watchpoint(&mut self, watch_point: &PositionWatchpointData) -> Result<bool> {
        Ok(self.inner.add_position_watchpoint(watch_point))
    }

    pub fn remove_position_watchpoint(&mut self, watch_point: &PositionWatchpointData) -> Result<bool> {
        Ok(self.inner.remove_position_watchpoint(watch_point))
    }

    pub fn set_replay_progress_callback(&mut self, cb: ReplayProgressCallback) {
        let ptr = cb as *mut sys::ReplayProgressCallbackUnsafe;
        self.inner.set_replay_progress_callback(unsafe { *ptr });
    }

    pub fn set_register_changed_callback(&mut self, cb: RegisterChangedCallback) {
        let ptr = cb as *mut sys::RegisterChangedCallbackUnsafe;
        self.inner.set_register_changed_callback(unsafe { *ptr });
    }
}

pub struct ReplayEngine {
    inner: crate::replay::sys::ReplayEngine,
}

impl ReplayEngine {
    pub fn new() -> Result<Self> {
        Ok(Self {
            inner: crate::replay::sys::ReplayEngine::new()?,
        })
    }

    pub fn load(&self, trace_path: &std::path::Path) -> Result<()> {
        if !trace_path.exists() {
            return Err(Error::NotFound);
        }

        let c_str = CString::from_str(trace_path.to_str().ok_or(Error::ConversionError)?)?;
        let w_str: Vec<u16> = c_str.to_string_lossy().encode_utf16().chain(std::iter::once(0)).collect();
        match self.inner.load(&w_str) {
            0 => Ok(()),
            _ => Err(Error::InitializationError),
        }
    }

    /// The proper way to get a new cursor for the replay engine.
    pub fn cursor(&'_ self) -> Result<ReplayCursor<'_>> {
        Ok(ReplayCursor { inner: self.inner.cursor()? })
    }

    pub fn get_lifetime(&self) -> &PositionRange {
        self.inner.get_lifetime()
    }

    pub fn build_index(&self) -> Result<()> {
        match self.inner.build_index() {
            0 => Ok(()),
            _ => Err(Error::ForeignFunctionError),
        }
    }

    pub fn system_info(&self) -> Result<&SystemInfo> {
        Ok(self.inner.system_info())
    }

    pub fn process_id(&self) -> Result<u32> {
        Ok(self.system_info()?.ProcessId)
    }

    pub fn get_module_count(&self) -> Result<usize> {
        Ok(self.inner.get_module_count())
    }

    pub fn get_module_list(&self) -> Result<Vec<ReplayModule>> {
        let mut res = Vec::<ReplayModule>::with_capacity(self.get_module_count()?);
        for module in self.inner.get_module_list().iter() {
            res.push(module.try_into()?);
        }
        Ok(res)
    }

    pub fn get_thread_count(&self) -> Result<usize> {
        Ok(self.inner.get_thread_count())
    }

    pub fn get_thread_list(&self) -> Result<Vec<ThreadInfo>> {
        Ok(self.inner.get_thread_list())
    }

    pub fn get_module_loaded_event_count(&self) -> Result<usize> {
        Ok(self.inner.get_module_loaded_event_count())
    }

    pub fn get_module_loaded_event_list(&self) -> Result<Vec<events::ModuleLoaded>> {
        let mut res = Vec::<events::ModuleLoaded>::with_capacity(self.get_module_loaded_event_count()?);
        for module in self.inner.get_module_loaded_event_list().iter() {
            res.push(module.try_into()?);
        }
        Ok(res)
    }

    pub fn get_module_unloaded_event_count(&self) -> Result<usize> {
        Ok(self.inner.get_module_unloaded_event_count())
    }

    pub fn get_module_unloaded_event_list(&self) -> Result<Vec<events::ModuleUnloaded>> {
        let mut res = Vec::<events::ModuleUnloaded>::with_capacity(self.get_module_unloaded_event_count()?);
        for module in self.inner.get_module_unloaded_event_list().iter() {
            res.push(module.try_into()?);
        }
        Ok(res)
    }

    pub fn get_exception_event_count(&self) -> Result<usize> {
        Ok(self.inner.get_exception_event_count())
    }

    pub fn get_exception_event_list(&self) -> Result<Vec<events::Exception>> {
        let mut res = Vec::<events::Exception>::with_capacity(self.get_exception_event_count()?);
        for module in self.inner.get_exception_event_list().iter() {
            res.push(module.try_into()?);
        }
        Ok(res)
    }

    pub fn get_module_base_address(&self, module_name: &str) -> Result<u64> {
        let mod_lower = module_name.to_lowercase();
        let modules = self.get_module_loaded_event_list()?;
        let matches: Vec<&events::ModuleLoaded> = modules.iter().filter(|e| e.module.name.to_lowercase().ends_with(&mod_lower)).collect();
        if matches.len() > 1 {
            return Err(Error::DataMismatch);
        }

        Ok(matches.first().ok_or(Error::NotFound)?.module.address)
    }
}
// endregion: TTD Replay

#[cfg(test)]
mod test {
    use crate::{
        bindings::root::TTD_FFI::Replay::{MAX_CURSOR, MAX_ENGINE},
        replay::{EngineInfo, EventType, ReplayCursor, ReplayEngine},
    };

    fn get_test_trace() -> std::path::PathBuf {
        let mut trace_path = std::path::PathBuf::from(std::env::var("TEMP").expect("failed to get TEMP env var").as_str());
        trace_path.push("test.run");
        trace_path
    }

    #[test]
    fn test_ffi_version() {
        let info = EngineInfo::new();
        assert_eq!((info.major, info.minor, info.patch), (0, 1, 0));
        assert_ne!(info.license.len(), 0);
        assert_ne!(info.author.len(), 0);
        assert_ne!(info.banner.len(), 0);
        assert_ne!(info.name.len(), 0);
    }

    #[test]
    fn test_ffi_load_simple() {
        let mut engine = ReplayEngine::new().expect("failed to create a new replayer");

        let trace_path = get_test_trace();
        assert!(engine.load(trace_path.as_path()).is_ok());

        for i in 1..10 {
            let mut cursor = engine.cursor().unwrap();
            assert_eq!(*cursor.get_position().unwrap(), engine.get_lifetime().Min);

            cursor.set_position(&engine.get_lifetime().Max);
            assert_eq!(*cursor.get_position().unwrap(), engine.get_lifetime().Max);

            cursor.set_position(&engine.get_lifetime().Min);
            assert_eq!(*cursor.get_position().unwrap(), engine.get_lifetime().Min);
        }

        for i in 1..10 {
            let mut cursor = engine.cursor().unwrap();
            assert_eq!(*cursor.get_position().unwrap(), engine.get_lifetime().Min);

            let res = cursor.replay_forward(None).unwrap();
            assert_eq!(res.stop_reason, EventType::Process);
            assert_ne!(res.instructions_executed, 0);
            assert_eq!(*cursor.get_previous_position().unwrap(), engine.get_lifetime().Max);

            let res = cursor.replay_backward(None).unwrap();
            assert_eq!(res.stop_reason, EventType::Process);
            assert_ne!(res.instructions_executed, 0);
            assert_eq!(*cursor.get_position().unwrap(), engine.get_lifetime().Min);
        }
    }

    #[test]
    fn test_ffi_system_info() {
        let engine = ReplayEngine::new().expect("failed to create a new replayer");

        let trace_path = get_test_trace();
        assert!(engine.load(trace_path.as_path()).is_ok());

        let info = engine.system_info().unwrap();
        assert_eq!(info.SystemName.len(), 64);
        assert_eq!(info.UserName.len(), 64);
    }
}
