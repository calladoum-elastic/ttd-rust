use derive_more::Display;

use crate::prelude::*;
use crate::replay::{ReplayModule, ReplayPosition};

#[derive(Debug)]
pub struct ModuleLoaded {
    pub position: ReplayPosition,
    pub module: ReplayModule,
}

impl TryFrom<&ttd_sys::bindings::root::TTD::Replay::ModuleLoadedEvent> for ModuleLoaded {
    fn try_from(value: &ttd_sys::bindings::root::TTD::Replay::ModuleLoadedEvent) -> Result<Self> {
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
impl TryFrom<&ttd_sys::bindings::root::TTD::Replay::ModuleUnloadedEvent> for ModuleUnloaded {
    fn try_from(value: &ttd_sys::bindings::root::TTD::Replay::ModuleUnloadedEvent) -> Result<Self> {
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
impl TryFrom<&ttd_sys::bindings::root::TTD::Replay::ExceptionEvent> for Exception {
    fn try_from(value: &ttd_sys::bindings::root::TTD::Replay::ExceptionEvent) -> Result<Self> {
        todo!()
    }
    type Error = crate::error::Error;
}
