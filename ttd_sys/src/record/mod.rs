#![allow(unused)]

use crate::prelude::*;

use crate::bindings::root as ffi;

pub struct Recorder<'a> {
    inner: ffi::TTD_FFI::Record::ScopedRecorder,
    engine: &'a RecorderEngine,
}

impl<'a> Recorder<'a> {
    /// ## Description
    /// Start recording
    pub fn start(&'a self) {
        unsafe {
            self.inner.Start();
        }
    }

    /// ## Description
    /// Stop recording
    pub fn stop(&'a self) {
        unsafe {
            self.inner.Stop();
        }
    }
}

pub struct RecorderEngine {
    name: String,
    inner: ffi::TTD_FFI::Record::RecorderEngine,
}

impl RecorderEngine {
    /// ## Description
    /// Initialize a new [`RecorderEngine`]
    ///
    pub fn new(name: &[u8]) -> Result<Self> {
        Ok(Self {
            inner: unsafe { ffi::TTD_FFI::Record::RecorderEngine::new(name.as_ptr()) },
            name: std::str::from_utf8(name)?.to_string(),
        })
    }

    /// ## Description
    /// Create a new recorder
    ///
    /// ## Returns
    ///  - A [`Result`] a result to [`Recorder`] whose lifetime is tied to this [`RecorderEngine`]
    pub fn recorder(&'_ self) -> Result<Recorder<'_>> {
        let recorder = unsafe { self.inner.Recorder() };
        Ok(Recorder { inner: recorder, engine: self })
    }

    /// ## Description
    /// Save content into the file path given in arguments
    ///
    /// ## Parameters
    ///  - `path` a mutable slice that will store the recorded trace path
    ///
    /// ## Returns
    ///  - bool : `true` on success
    ///
    /// ## Safety
    /// Calls into FFI `TTD_FFI::Record::RecorderEngine::Save`
    pub fn save(&self, path: &mut [u16]) -> bool {
        unsafe { self.inner.Save(path.as_ptr(), path.len() as u64) }
    }
}

impl std::fmt::Display for RecorderEngine {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "RecorderEngine({})", self.name)
    }
}

impl std::fmt::Debug for RecorderEngine {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RecorderEngine").field("name", &self.name).finish_non_exhaustive()
    }
}

#[cfg(test)]
mod test {
    use crate::record::{Recorder, RecorderEngine};

    #[test]
    #[ignore = "Record API seems buggy for now"]
    fn record_self() {
        let eng = RecorderEngine::new("TestRecorder".as_ref()).unwrap();
        let rec = eng.recorder().unwrap();

        rec.start();
        for _i in 1..10_000 {}
        rec.stop();

        let mut trace_path = [0u16; 1024];
        eng.save(&mut trace_path);
    }
}
