//! Contains the replay module
// Most operations here are exposed as safe
//
use derive_more::Display;

use crate::prelude::*;

pub(crate) mod sys;

use std::{
    ffi::CString,
    ops::{Add, Sub},
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
pub type RegisterContext = crate::replay::sys::RegisterContext;
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

#[derive(Default, Debug)]
pub struct ReplayEngine {}

impl ReplayEngine {
    pub fn new() -> Result<Self> {
        if sys::initialize() != 0 {
            return Err(Error::InitializationError);
        }

        Ok(Self {})
    }

    pub fn load(&self, trace_path: &std::path::Path) -> Result<()> {
        if !trace_path.exists() {
            return Err(Error::NotFound);
        }

        let c_str = CString::from_str(trace_path.to_str().ok_or(Error::ConversionError)?)?;
        match sys::load(c_str.to_str()?) {
            0 => Ok(()),
            _ => Err(Error::InitializationError),
        }
    }

    pub fn build_index(&self) -> Result<u32> {
        Ok(sys::build_index())
    }

    pub fn system_info(&self) -> Result<SystemInfo> {
        Ok(sys::system_info())
    }

    pub fn process_id(&self) -> Result<u32> {
        Ok(sys::system_info().ProcessId)
    }

    pub fn replay_forward(&self, until: Option<ReplayPosition>) -> Result<ReplayResult> {
        Ok(sys::replay_forward(until).into())
    }

    pub fn replay_backward(&self, until: Option<ReplayPosition>) -> Result<ReplayResult> {
        Ok(sys::replay_backward(until).into())
    }

    pub fn replay_forward_steps(&self, steps: u64) -> Result<ReplayResult> {
        let until = self.get_position()? + steps;
        Ok(sys::replay_forward(Some(until)).into())
    }

    pub fn replay_backward_steps(&self, steps: u64) -> Result<ReplayResult> {
        let until = self.get_position()? + steps;
        Ok(sys::replay_backward(Some(until)).into())
    }

    pub fn set_replay_progress_callback(&self, cb: ReplayProgressCallback) {
        let ptr = cb as *mut sys::ReplayProgressCallbackUnsafe;
        sys::set_replay_progress_callback(unsafe { *ptr });
    }

    pub fn set_register_changed_callback(&self, cb: RegisterChangedCallback) {
        let ptr = cb as *mut sys::RegisterChangedCallbackUnsafe;
        sys::set_register_changed_callback(unsafe { *ptr });
    }

    pub fn set_position(&self, pos: &ReplayPosition) {
        sys::set_position(pos);
    }

    pub fn get_position(&self) -> Result<ReplayPosition> {
        Ok(sys::get_position())
    }

    pub fn pointer_size(&self) -> Result<usize> {
        match sys::get_thread_context()? {
            sys::RegisterContext::X64(amd64_context) => Ok(8),
            sys::RegisterContext::X86(x86_nt5_context) => Ok(4),
        }
    }

    pub fn get_thread_context(&self) -> Result<RegisterContext> {
        sys::get_thread_context()
    }

    pub fn get_thread_extended_context(&self) {}

    pub fn read_current_memory(&self, address: u64, size: usize) -> Result<Vec<u8>> {
        sys::read_current_memory(address, size)
    }

    pub fn get_thread_info(&self) -> Result<ThreadInfo> {
        Ok(sys::get_thread_info())
    }

    pub fn get_previous_position(&self) -> Result<ReplayPosition> {
        Ok(sys::get_previous_position())
    }

    pub fn get_teb_address(&self) -> Result<u64> {
        Ok(sys::get_teb_address())
    }

    pub fn get_program_counter(&self) -> Result<u64> {
        Ok(sys::get_program_counter())
    }

    pub fn get_stack_pointer(&self) -> Result<u64> {
        Ok(sys::get_stack_pointer())
    }

    pub fn get_frame_pointer(&self) -> Result<u64> {
        Ok(sys::get_frame_pointer())
    }

    pub fn get_replay_flags(&self) -> Result<ReplayFlags> {
        Ok(sys::get_replay_flags())
    }

    pub fn set_replay_flags(&self, flags: ReplayFlags) -> Result<()> {
        sys::set_replay_flags(flags);
        Ok(())
    }

    pub fn add_memory_watchpoint(&self, watch_point: &MemoryWatchpointData) -> Result<bool> {
        Ok(sys::add_memory_watchpoint(watch_point))
    }

    pub fn remove_memory_watchpoint(&self, watch_point: &MemoryWatchpointData) -> Result<bool> {
        Ok(sys::remove_memory_watchpoint(watch_point))
    }

    pub fn add_position_watchpoint(&self, watch_point: &PositionWatchpointData) -> Result<bool> {
        Ok(sys::add_position_watchpoint(watch_point))
    }

    pub fn remove_position_watchpoint(&self, watch_point: &PositionWatchpointData) -> Result<bool> {
        Ok(sys::remove_position_watchpoint(watch_point))
    }

    pub fn get_module_count(&self) -> Result<usize> {
        Ok(sys::get_module_count())
    }

    pub fn get_module_list(&self) -> Result<Vec<ReplayModule>> {
        let mut res = Vec::<ReplayModule>::with_capacity(self.get_module_count()?);
        for module in sys::get_module_list().iter() {
            res.push(module.try_into()?);
        }
        Ok(res)
    }

    pub fn get_thread_count(&self) -> Result<usize> {
        Ok(sys::get_thread_count())
    }

    pub fn get_thread_list(&self) -> Result<Vec<ThreadInfo>> {
        Ok(sys::get_thread_list())
    }

    pub fn get_module_loaded_event_count(&self) -> Result<usize> {
        Ok(sys::get_module_loaded_event_count())
    }

    pub fn get_module_loaded_event_list(&self) -> Result<Vec<events::ModuleLoaded>> {
        let mut res = Vec::<events::ModuleLoaded>::with_capacity(self.get_module_loaded_event_count()?);
        for module in sys::get_module_loaded_event_list().iter() {
            res.push(module.try_into()?);
        }
        Ok(res)
    }

    pub fn get_module_unloaded_event_count(&self) -> Result<usize> {
        Ok(sys::get_module_unloaded_event_count())
    }

    pub fn get_module_unloaded_event_list(&self) -> Result<Vec<events::ModuleUnloaded>> {
        let mut res = Vec::<events::ModuleUnloaded>::with_capacity(self.get_module_unloaded_event_count()?);
        for module in sys::get_module_unloaded_event_list().iter() {
            res.push(module.try_into()?);
        }
        Ok(res)
    }

    pub fn get_exception_event_count(&self) -> Result<usize> {
        Ok(sys::get_exception_event_count())
    }

    pub fn get_exception_event_list(&self) -> Result<Vec<events::Exception>> {
        let mut res = Vec::<events::Exception>::with_capacity(self.get_exception_event_count()?);
        for module in sys::get_exception_event_list().iter() {
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

impl Drop for ReplayEngine {
    fn drop(&mut self) {
        let _ = sys::reset();
    }
}

// endregion: TTD Replay

#[cfg(test)]
mod test {
    use crate::replay::{EngineInfo, ReplayEngine};

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
    fn test_ffi_replay_basic() {
        let replay = ReplayEngine::new().expect("failed to create a new replay");
        assert!(replay.process_id().is_ok());

        let trace_path = std::path::Path::new("c:\\users\\chris\\documents\\notepad03.run");
        assert!(replay.load(trace_path).is_ok());

        let info = replay.system_info().expect("system_info() failed");

        assert_ne!(info.SystemName.len(), 0);
        assert_ne!(info.UserName.len(), 0);
        assert_ne!(info.UserName.len(), 0);
    }
}
