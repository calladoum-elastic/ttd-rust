//! Contains the replay module
// Most operations here are exposed as safe
//
use derive_more::Display;

use ttd_sys as sys;
use ttd_sys::bindings;

use crate::prelude::*;

use std::ffi::CString;
use std::ops::{Add, Sub};
use std::os::windows::ffi::OsStringExt;
use std::str::FromStr;

use bitflags::bitflags;

pub mod events;

pub type SystemInfo = bindings::root::TTD::SystemInfo;
pub type ThreadInfo = bindings::root::TTD::Replay::ThreadInfo;
// pub type ReplayPosition = bindings::root::TTD::Replay::Position;
pub type ReplayPositionRange = bindings::root::TTD::Replay::PositionRange;
pub type ThreadView = bindings::root::TTD::Replay::IThreadView;
pub type Amd64Context = bindings::root::AMD64_CONTEXT;
pub type Amd64ExtendedContext = bindings::root::AVX_EXTENDED_CONTEXT;

pub type ReplayFlags = ttd_sys::replay::ReplayFlags;
pub type RegisterContext<'a> = ttd_sys::replay::RegisterContext<'a>;
pub type ExtendedRegisterContext = ttd_sys::replay::ExtendedRegisterContext;

pub type SequenceId = bindings::root::TTD::SequenceId;
pub type PositionWatchpointData = bindings::root::TTD::Replay::PositionWatchpointData;
pub type MemoryWatchpointData = bindings::root::TTD::Replay::MemoryWatchpointData;

pub type ReplayProgressCallback = fn(ctx: usize, pos: &ReplayPosition);
pub type RegisterChangedCallback = fn(context: usize, reg_id: u8, old_data: &[u8; 8], new_data: &[u8; 8], ddata_size_in_bytes: usize, thread: &ThreadView);

/// Wrapper around the raw TTD SDK Replay::Position reference that provides a
/// borrowed, ergonomic handle to a Time-Travel Debugging replay position
/// without taking ownership. The lifetime 'a ties the wrapper to the
/// referenced bindings::root::TTD::Replay::Position, preventing the wrapper
/// from outliving the underlying SDK object.
///
/// Parameters:
/// - 'a: Lifetime of the borrowed bindings::root::TTD::Replay::Position,
///   ensuring the wrapper does not outlive the referenced SDK value.
#[derive(Display, Debug, PartialEq)]
pub struct ReplayPosition<'a>(&'a bindings::root::TTD::Replay::Position);

impl<'a> Into<&'a bindings::root::TTD::Replay::Position> for ReplayPosition<'a> {
    fn into(self) -> &'a bindings::root::TTD::Replay::Position {
        self.0
    }
}

impl<'a> From<&'a bindings::root::TTD::Replay::Position> for ReplayPosition<'a> {
    fn from(value: &'a bindings::root::TTD::Replay::Position) -> Self {
        Self(value)
    }
}

/// Represents a module (loaded binary or library) observed during a TTD record
/// or replay session. Encapsulates metadata such as module base address,
/// size, file path, timestamp/version info, and identifiers used by the TTD
/// SDK to correlate module load/unload events. Use this struct to inspect
/// which modules were present at specific replay positions, resolve symbols,
/// or present module lists to users.
#[derive(Debug)]
pub struct ReplayModule {
    pub name: String,
    pub address: u64,
    pub size: u64,
    pub checksum: u32,
    pub timestamp: u32,
}

impl TryFrom<&bindings::root::TTD::Replay::Module> for ReplayModule {
    fn try_from(value: &bindings::root::TTD::Replay::Module) -> Result<Self> {
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

/// Represents a specific in-process instantiation of a module observed during a
/// TTD record or replay session. Contains instance-specific details such as the
/// module's load base address, applied relocation or ASLR offset, instance lifetime
/// (load/unload positions), and identifiers tying the instance back to the
/// canonical module metadata. Use this struct to track which particular module
/// image was active on a thread or at a replay position, to resolve addresses
/// to module-relative offsets, and to correlate instance-level events.
#[derive(Debug)]
pub struct ModuleInstance {
    /// The associated [`ReplayModule`]
    pub module: ReplayModule,
    /// The module load timestamp
    pub load_time: SequenceId,
    /// The module unload timestamp
    pub unload_time: SequenceId,
}

#[derive(Debug)]
pub struct ActiveThreadInfo {
    pub thread: ThreadInfo,
    pub current_position: bindings::root::TTD::Replay::Position,
    pub last_valid_position: bindings::root::TTD::Replay::Position,
}

/// Holds the outcome of a replay step or operation in ttd-rust, describing why
/// replay stopped and quantitative execution metrics. Useful for callers that
/// need to inspect the stop cause and how much work the replay performed.
pub struct ReplayResult {
    /// The [`events.EventType`] of the reason why the replay stopped
    pub stop_reason: events.EventType,

    /// Indicates how many steps were ran
    pub steps_executed: u64,

    /// Indicates how many instructions were ran
    pub instructions_executed: u64,
}

impl From<bindings::root::TTD::Replay::ICursorView_ReplayResult> for ReplayResult {
    fn from(value: bindings::root::TTD::Replay::ICursorView_ReplayResult) -> Self {
        Self {
            stop_reason: value.StopReason.into(),
            steps_executed: value.StepsExecuted,
            instructions_executed: value.InstructionsExecuted,
        }
    }
}

// region: TTD Replay Cursor

/// A high-level, borrow-based cursor over a TTD replay stream that wraps the
/// lower-level FFI crate::replay::sys::replay::ReplayCursor. Provides methods
/// to navigate replay events and positions while preserving Rust lifetime safety
/// for referenced SDK state. Use this type to iterate or seek through recorded
/// execution without taking ownership of the underlying replay session.
pub struct ReplayCursor<'a> {
    inner: crate::replay::sys::replay::ReplayCursor<'a>,
}

impl<'a> ReplayCursor<'a> {

    /// Advance the cursor forward toward an optional target position. If until is
    /// `Some`, replay proceeds until that `ReplayPosition` or a stopping event;
    // if `None`, it advances until another event raised. On success the function
    /// returns `ReplayResult` indicating the stop reason and other metrics.
    ///
    /// Parameters:
    /// - `until`: Optional target [`ReplayPosition`] to stop at.
    ///
    /// Returns:
    /// - [`Result<ReplayResult>`]
    pub fn replay_forward(&mut self, until: Option<ReplayPosition>) -> Result<ReplayResult> {
        Ok(match until {
            Some(pos) => self.inner.replay_forward(Some(*pos.0))?,
            None => self.inner.replay_forward(None)?,
        }
        .into())
    }

    /// Same as `replay_forward()` but backward
    ///
    /// Parameters:
    /// - `until`: Optional target [`ReplayPosition`] to stop at.
    ///
    /// Returns:
    /// - [`Result`]
    pub fn replay_backward(&mut self, until: Option<ReplayPosition>) -> Result<ReplayResult> {
        Ok(match until {
            Some(pos) => self.inner.replay_backward(Some(*pos.0))?,
            None => self.inner.replay_backward(None)?,
        }
        .into())
    }

    /// Advances forward the replay by a specific number of `steps`
    ///
    /// Parameters:
    /// - `step`: The number of steps to move forward.
    ///
    /// Returns:
    /// - [`Result<ReplayResult>`]
    pub fn replay_forward_steps(&mut self, steps: u64) -> Result<ReplayResult> {
        let until = *self.inner.get_position() + steps;
        Ok(self.inner.replay_forward(Some(until))?.into())
    }

    /// Advances backward the replay by a specific number of `steps`
    ///
    /// Parameters:
    /// - `step`: The number of steps to move backward.
    ///
    /// Returns:
    /// - [`Result<ReplayResult>`]
    pub fn replay_backward_steps(&mut self, steps: u64) -> Result<ReplayResult> {
        let until = *self.inner.get_position() + steps;
        Ok(self.inner.replay_backward(Some(until))?.into())
    }

    /// Set the cursor to the given replay position immediately. `set_position()`
    ///  simply places the cursor at the intended `ReplayPosition`. As such it
    /// will ignore any event occuring between the old and new position.
    ///
    /// Parameters:
    /// - `pos`: Reference to the target ReplayPosition to set.
    pub fn set_position(&mut self, pos: &ReplayPosition) {
        self.inner.set_position(pos.0)
    }

    /// Get the current position of the cursor.
    ///
    /// Returns:
    /// - [`Result<ReplayPosition>`]
    pub fn get_position(&self) -> Result<ReplayPosition<'_>> {
        Ok(ReplayPosition(self.inner.get_position()))
    }

    /// Get the previous position of the cursor.
    ///
    /// Returns:
    /// - [`Result<ReplayPosition>`]
    pub fn get_previous_position(&mut self) -> Result<ReplayPosition<'_>> {
        Ok(ReplayPosition(self.inner.get_previous_position()))
    }

    /// Get the thread information at the current point of replay as a
    /// reference to [`ThreadInfo`].
    ///
    /// Returns:
    /// - [`Result<&ThreadInfo>`]
    pub fn get_thread_info(&self) -> Result<&ThreadInfo> {
        Ok(self.inner.get_thread_info())
    }

    /// Get the [TEB](https://www.geoffchappell.com/studies/windows/km/ntoskrnl/inc/api/pebteb/teb/index.htm)
    /// address of the current active thread.
    ///
    /// Returns:
    /// - [`Result<u64>`]
    pub fn get_teb_address(&self) -> Result<u64> {
        Ok(self.inner.get_teb_address())
    }

    /// Get the current PC value. Equivalent to getting the PC value
    /// from [`get_thread_context()`]
    ///
    /// Returns:
    /// - [`Result<u64>`]
    pub fn get_program_counter(&self) -> Result<u64> {
        Ok(self.inner.get_program_counter())
    }

    /// Get the current SP value. Equivalent to getting the SP value
    /// from [`get_thread_context()`]
    ///
    /// Returns:
    /// - [`Result<u64>`]
    pub fn get_stack_pointer(&self) -> Result<u64> {
        Ok(self.inner.get_stack_pointer())
    }

    /// Get the current FP value. Equivalent to getting the FP value
    /// from [`get_thread_context()`]
    ///
    /// Returns:
    /// - [`Result<u64>`]
    pub fn get_frame_pointer(&self) -> Result<u64> {
        Ok(self.inner.get_frame_pointer())
    }

    /// Get the current [`RegisterContext`] with the state of all registers at
    /// the current point of execution.
    ///
    /// Returns:
    /// - [`Result<RegisterContext>`]
    pub fn get_thread_context(&self) -> Result<RegisterContext<'_>> {
        Ok(self.inner.get_thread_context()?)
    }

    /// Get the size of pointer for the current architecture.
    ///
    /// Returns:
    /// - [`Result<usize>`]
    pub fn pointer_size(&self) -> Result<usize> {
        match self.get_thread_context()? {
            ttd_sys::replay::RegisterContext::X64(_) => Ok(8),
            ttd_sys::replay::RegisterContext::X86(_) => Ok(4),
            ttd_sys::replay::RegisterContext::ARM64(_) => Ok(8),
        }
    }

    /// Get the current [`ExtendedRegisterContext`] with the state of all
    /// extended registers at the current point of execution.
    ///
    /// Returns:
    /// - [`Result<ExtendedRegisterContext>`]
    pub fn get_thread_extended_context(&self) {
        unimplemented!()
    }

    /// Read size bytes from the replay's current memory state at address and
    /// return them as a byte vector. This observes the memory view at the cursor's
    /// current position.
    ///
    /// Parameters:
    /// - address: Starting virtual address to read from.
    /// - size: Number of bytes to read.
    ///
    /// Returns:
    /// - Result<Vec>
    pub fn read_current_memory(&self, address: u64, size: usize) -> Result<Vec<u8>> {
        Ok(self.inner.read_current_memory(address, size)?)
    }

    pub fn get_replay_flags(&self) -> Result<ReplayFlags> {
        Ok(self.inner.get_replay_flags())
    }

    pub fn set_replay_flags(&mut self, flags: ReplayFlags) {
        self.inner.set_replay_flags(flags);
    }

    /// Add a memory watchpoint that triggers when the specified memory region is
    /// accessed during replay. Returns true if the watchpoint was added; false if
    /// it was ignored or already present.
    ///
    /// Parameters:
    /// - watch_point: Reference to [`MemoryWatchpointData`] describing address, size, and access type.
    ///
    /// Returns:
    /// - [`Result<bool>`]
    pub fn add_memory_watchpoint(&mut self, watch_point: &MemoryWatchpointData) -> Result<bool> {
        Ok(self.inner.add_memory_watchpoint(watch_point))
    }

    /// Remove a memory watchpoint that triggers when the specified memory region is
    /// accessed during replay. Returns true if the watchpoint was added; false if
    /// it was ignored or already present.
    ///
    /// Parameters:
    /// - watch_point: Reference to [`MemoryWatchpointData`] describing address, size, and access type.
    ///
    /// Returns:
    /// - [`Result<bool>`]
    pub fn remove_memory_watchpoint(&mut self, watch_point: &MemoryWatchpointData) -> Result<bool> {
        Ok(self.inner.remove_memory_watchpoint(watch_point))
    }

    /// Add a position-based watchpoint that triggers when the replay reaches the
    /// specified ReplayPosition or meets its conditions. Returns true if the
    /// watchpoint was registered, false if ignored or duplicated.
    ///
    /// Parameters:
    /// - watch_point: Reference to [`PositionWatchpointData`] specifying the target position and trigger criteria.
    ///
    /// Returns:
    /// - [`Result<bool>`]
    pub fn add_position_watchpoint(&mut self, watch_point: &PositionWatchpointData) -> Result<bool> {
        Ok(self.inner.add_position_watchpoint(watch_point))
    }

    /// Remove a position-based watchpoint that triggers when the replay reaches the
    /// specified ReplayPosition or meets its conditions. Returns true if the
    /// watchpoint was registered, false if ignored or duplicated.
    ///
    /// Parameters:
    /// - watch_point: Reference to [`PositionWatchpointData`] specifying the target position and trigger criteria.
    ///
    /// Returns:
    /// - [`Result<bool>`]
    pub fn remove_position_watchpoint(&mut self, watch_point: &PositionWatchpointData) -> Result<bool> {
        Ok(self.inner.remove_position_watchpoint(watch_point))
    }

    pub fn set_replay_progress_callback(&mut self, cb: ReplayProgressCallback) {
        let ptr = cb as *mut ttd_sys::replay::ReplayProgressCallbackUnsafe;
        self.inner.set_replay_progress_callback(unsafe { *ptr });
    }

    pub fn set_register_changed_callback(&mut self, cb: RegisterChangedCallback) {
        let ptr = cb as *mut ttd_sys::replay::RegisterChangedCallbackUnsafe;
        self.inner.set_register_changed_callback(unsafe { *ptr });
    }
}
// endregion: TTD Replay Cursor

// region: TTD Replay Engine

/// High-level handle to the underlying TTD replay engine, wrapping the raw
/// FFI `ttd_sys::replay::ReplayEngine`. Manages replay session loading, configuration,
/// and creation of cursors while encapsulating ownership and resource cleanup.
pub struct ReplayEngine {
    inner: ttd_sys::replay::ReplayEngine,
}

impl ReplayEngine {
    pub fn new() -> Result<Self> {
        Ok(Self {
            inner: ttd_sys::replay::ReplayEngine::new()?,
        })
    }

    /// Load a TTD trace from the filesystem into the replay engine, preparing it
/// for cursor creation and navigation. This does not start replaying; it
/// initializes internal state from the specified trace file or directory.
///
/// Parameters:
/// - trace_path: Filesystem path to the TTD trace file or trace directory.
///
/// Returns:
/// - `Result`
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

    pub fn get_lifetime(&self) -> &ReplayPositionRange {
        self.inner.get_lifetime()
    }

    pub fn build_index(&self) -> Result<()> {
        match self.inner.build_index() {
            0 => Ok(()),
            _ => Err(Error::ForeignFunctionError),
        }
    }

    /// Retrieve system information captured by the loaded TTD trace (CPU, OS
    /// version, address width, endianness, and other environment details) part of the [`SystemInfo`] structure.
    ///
    /// Returns:
    /// - [`Result<&SystemInfo>`]
    pub fn system_info(&self) -> Result<&SystemInfo> {
        Ok(self.inner.system_info())
    }

    /// A convenience function leveraging `system_info()` to return the process id
    ///
    /// Returns:
    /// - [`Result<u32>`]
    pub fn process_id(&self) -> Result<u32> {
        Ok(self.system_info()?.ProcessId)
    }

    pub fn get_module_count(&self) -> Result<usize> {
        Ok(self.inner.get_module_count())
    }

    /// Return the list of modules observed in the loaded TTD trace as a vector of
    /// ReplayModule entries. Each element contains metadata (base address, size,
    /// path, version/timestamp) for a module recorded during execution.
    ///
    /// Returns:
    /// - `Result<Vec<ReplayModule>>`
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
// endregion: TTD Replay Engine

#[cfg(test)]
mod test {
    use std::ops::Add;

    use ttd_sys::bindings::root::TTD::Replay::PositionWatchpointData;

    use crate::prelude::*;
    use crate::replay::{DataAccessMask, events.EventType, MemoryWatchpointData, ReplayCursor, ReplayEngine, ReplayPosition};

    fn get_test_trace() -> std::path::PathBuf {
        let mut trace_path = std::path::PathBuf::from(std::env::var("TEMP").expect("failed to get TEMP env var").as_str());
        trace_path.push("test.run");
        trace_path
    }

    #[test]
    fn test_load_simple() {
        let mut engine = ReplayEngine::new().expect("failed to create a new replayer");

        let trace_path = get_test_trace();
        assert!(engine.load(trace_path.as_path()).is_ok());

        for i in 1..10 {
            let mut cursor = engine.cursor().unwrap();
            let curpos = cursor.get_position().unwrap();
            assert_eq!(*curpos.0, engine.get_lifetime().Min);

            let new_pos = ReplayPosition(&engine.get_lifetime().Max);
            cursor.set_position(&new_pos);
            let curpos = cursor.get_position().unwrap();
            assert_eq!(*curpos.0, engine.get_lifetime().Max);

            let new_pos = ReplayPosition(&engine.get_lifetime().Min);
            cursor.set_position(&new_pos);
            let curpos = cursor.get_position().unwrap();
            assert_eq!(*curpos.0, engine.get_lifetime().Min);
        }

        for i in 1..10 {
            let mut cursor = engine.cursor().unwrap();
            assert_eq!(*cursor.get_position().unwrap().0, engine.get_lifetime().Min);

            let res = cursor.replay_forward(None).unwrap();
            assert_eq!(res.stop_reason, events.EventType::Process);
            assert_ne!(res.instructions_executed, 0);
            assert_eq!(*cursor.get_previous_position().unwrap().0, engine.get_lifetime().Max);

            let res = cursor.replay_backward(None).unwrap();
            assert_eq!(res.stop_reason, events.EventType::Process);
            assert_ne!(res.instructions_executed, 0);
        }
    }

    #[test]
    fn test_system_info() {
        let engine = ReplayEngine::new().expect("failed to create a new replayer");

        let trace_path = get_test_trace();

        let info = engine.system_info().unwrap();
        assert_eq!(info.SystemName.len(), 64);
        assert_eq!(info.UserName.len(), 64);
    }

    #[test]
    fn test_get_module_base_address() {
        let engine = ReplayEngine::new().expect("failed to create a new replayer");
        assert!(engine.load(get_test_trace().as_path()).is_ok());

        let testcases = ["ntdll.dll", "kernel32.dll", "kernelbase.dll"];

        // valid
        for tc in testcases {
            let res = engine.get_module_base_address(tc);
            assert!(res.is_ok_and(|v| v != 0));
        }

        // invalid
        let res = engine.get_module_base_address("___fooobar__.dll");
        assert!(res.is_err());
        assert!(matches!(res.unwrap_err(), Error::NotFound));
    }

    #[test]
    fn test_get_module_list() {
        let engine = ReplayEngine::new().expect("failed to create a new replay engine");
        let res = engine.get_module_list();
        assert!(res.is_ok());
        let val = res.unwrap();
        assert!(val.len() == 0);

        assert!(engine.load(get_test_trace().as_path()).is_ok());
        let res = engine.get_module_list();
        assert!(res.is_ok());
        let val = res.unwrap();
        assert!(val.len() > 0);
    }

    #[test]
    fn test_get_thread_list() {
        let engine = ReplayEngine::new().expect("failed to create a new replay engine");
        let res = engine.get_thread_list();
        assert!(res.is_ok());
        let val = res.unwrap();
        assert!(val.len() == 0);

        assert!(engine.load(get_test_trace().as_path()).is_ok());
        let res = engine.get_thread_list();
        assert!(res.is_ok());
        let val = res.unwrap();
        assert!(val.len() > 0);
    }

    #[test]
    fn test_get_module_loaded_event_list() {
        let engine = ReplayEngine::new().expect("failed to create a new replay engine");
        let res = engine.get_module_loaded_event_list();
        assert!(res.is_ok());
        let val = res.unwrap();
        assert!(val.len() == 0);

        assert!(engine.load(get_test_trace().as_path()).is_ok());
        let res = engine.get_module_loaded_event_list();
        assert!(res.is_ok());
        let val = res.unwrap();
        assert!(val.len() > 0);
    }

    #[test]
    fn test_get_module_unloaded_event_list() {
        let engine = ReplayEngine::new().expect("failed to create a new replay engine");
        let res = engine.get_module_unloaded_event_list();
        assert!(res.is_ok());
        let val = res.unwrap();
        assert!(val.len() == 0);

        assert!(engine.load(get_test_trace().as_path()).is_ok());
        let res = engine.get_module_unloaded_event_list();
        assert!(res.is_ok());
        let val = res.unwrap();
        assert!(val.len() > 0);
    }

    #[test]
    fn test_get_exception_event_list() {
        let engine = ReplayEngine::new().expect("failed to create a new replay engine");
        let res = engine.get_exception_event_list();
        assert!(res.is_ok());
        let val = res.unwrap();
        assert!(val.len() == 0);

        assert!(engine.load(get_test_trace().as_path()).is_ok());
        let res = engine.get_exception_event_list();
        assert!(res.is_ok());
    }

    #[test]
    fn test_replay_forward_backward_steps() {
        let engine = ReplayEngine::new().expect("failed to create a new replay engine");
        assert!(engine.load(get_test_trace().as_path()).is_ok());
        let mut cursor = engine.cursor().expect("failed to create a new cursor");

        for step in 1..10u64 {
            let curpos = cursor.get_position().unwrap();
            let res = cursor.replay_forward_steps(step).unwrap();
            assert_eq!(step, res.steps_executed);
            assert_eq!(res.stop_reason, events.EventType::Position);

            let res = cursor.replay_backward_steps(step).unwrap();
            assert_eq!(res.stop_reason, events.EventType::Position);
            assert_eq!(0, res.steps_executed);
        }
    }

    #[test]
    fn test_get_set_replay_flags() {
        let engine = ReplayEngine::new().expect("failed to create a new replay engine");
        assert!(engine.load(get_test_trace().as_path()).is_ok());
        let mut cursor = engine.cursor().expect("failed to create a new cursor");

        let mut flags: ttd_sys::replay::ReplayFlags = cursor.get_replay_flags().unwrap();
        flags = ttd_sys::replay::ReplayFlags::ReplaySegmentsSequentially;
        cursor.set_replay_flags(flags);
    }

    #[test]
    fn test_add_remove_memory_watchpoint() {
        let engine = ReplayEngine::new().expect("failed to create a new replay engine");
        assert!(engine.load(get_test_trace().as_path()).is_ok());
        let mut cursor = engine.cursor().expect("failed to create a new cursor");

        let pc = cursor.get_program_counter().unwrap();
        let next_pc = {
            let pos = cursor.get_position().unwrap().0.to_owned();
            cursor.set_position(&ReplayPosition(&(pos + 1)));

            let pc = cursor.get_program_counter().unwrap();

            let pos = cursor.get_position().unwrap().0.to_owned();
            cursor.set_position(&ReplayPosition(&(pos - 1)));
            pc
        };

        let watch = MemoryWatchpointData {
            Address: next_pc,
            Size: 1,
            AccessMask: DataAccessMask::Execute.bits(),
            ..Default::default()
        };
        assert!(cursor.add_memory_watchpoint(&watch).unwrap());
        let res = cursor.replay_forward(None).unwrap();
        assert_eq!(cursor.get_program_counter().unwrap(), next_pc);

        assert!(cursor.remove_memory_watchpoint(&watch).unwrap());
    }

    #[test]
    fn test_add_remove_position_watchpoint() {
        let engine = ReplayEngine::new().expect("failed to create a new replay engine");
        assert!(engine.load(get_test_trace().as_path()).is_ok());
        let mut cursor = engine.cursor().expect("failed to create a new cursor");

        let watch = PositionWatchpointData { ..Default::default() };
        assert!(cursor.add_position_watchpoint(&watch).unwrap());
        assert!(cursor.remove_position_watchpoint(&watch).unwrap());
    }

    #[test]
    fn test_get_thread_info() {
        let engine = ReplayEngine::new().expect("failed to create a new replay engine");
        assert!(engine.load(get_test_trace().as_path()).is_ok());
        let cursor = engine.cursor().expect("failed to create a new cursor");
        let thread_info = cursor.get_thread_info().unwrap();
        assert_ne!(thread_info.Id, 0);
        assert_ne!(thread_info.UniqueId, 0);
    }

    #[test]
    fn test_get_teb_address() {
        let engine = ReplayEngine::new().expect("failed to create a new replay engine");
        assert!(engine.load(get_test_trace().as_path()).is_ok());
        let cursor = engine.cursor().expect("failed to create a new cursor");
        let addr = cursor.get_teb_address().unwrap();
        assert_ne!(addr, 0);
    }

    #[test]
    fn test_get_program_counter() {
        let engine = ReplayEngine::new().expect("failed to create a new replay engine");
        assert!(engine.load(get_test_trace().as_path()).is_ok());
        let cursor = engine.cursor().expect("failed to create a new cursor");
        let program_counter = cursor.get_program_counter().unwrap();
        assert_ne!(program_counter, 0);
    }

    #[test]
    fn test_get_stack_pointer() {
        let engine = ReplayEngine::new().expect("failed to create a new replay engine");
        assert!(engine.load(get_test_trace().as_path()).is_ok());
        let cursor = engine.cursor().expect("failed to create a new cursor");
        let stack_pointer = cursor.get_stack_pointer().unwrap();
        assert_ne!(stack_pointer, 0);
    }

    #[test]
    fn test_get_frame_pointer() {
        let engine = ReplayEngine::new().expect("failed to create a new replay engine");
        assert!(engine.load(get_test_trace().as_path()).is_ok());
        let cursor = engine.cursor().expect("failed to create a new cursor");
        assert!(cursor.get_frame_pointer().is_ok());
    }

    #[test]
    fn test_get_thread_context() {
        let engine = ReplayEngine::new().expect("failed to create a new replay engine");
        assert!(engine.load(get_test_trace().as_path()).is_ok());
        let cursor = engine.cursor().expect("failed to create a new cursor");
        let thread_context = cursor.get_thread_context().unwrap();
        let (pc, sp) = match thread_context {
            ttd_sys::replay::RegisterContext::ARM64(ctx) => (ctx.Pc, ctx.Sp),
            ttd_sys::replay::RegisterContext::X64(ctx) => (ctx.Rip, ctx.Rsp),
            ttd_sys::replay::RegisterContext::X86(ctx) => (ctx.Eip as u64, ctx.Esp as u64),
        };
        assert_eq!(pc, cursor.get_program_counter().unwrap());
        assert_eq!(sp, cursor.get_stack_pointer().unwrap());
    }
}
