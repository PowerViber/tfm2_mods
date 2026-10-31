//! Observation only. Default build registers no callback. Never a realm backend.
use mod_api_stable::{HostApiV1, ModExportV1, StableHost, StableMod};

#[cfg(all(windows, feature = "observe-runtime"))]
mod observe {
    use super::*;
    use mod_api_stable::{MatchHookRegV1, MatchHookVtableV1, SimCtxV1, StableSim};
    use serde_json::{json, Value};
    use sha2::{Digest, Sha256};
    use std::{collections::BTreeMap, ffi::c_void, fs::OpenOptions, io::Write,
        mem::size_of, panic::{catch_unwind, AssertUnwindSafe}, path::PathBuf, sync::Mutex};
    const EXE_SHA: &str = "d678e01285b7a9edba2c7ee1067cea93a54f979d769eaae9f0d8309d18ff7d30";
    #[link(name = "kernel32")]
    extern "system" {
        fn GetModuleHandleW(name: *const u16) -> *mut c_void;
        fn GetCurrentProcess() -> *mut c_void;
        fn ReadProcessMemory(process: *mut c_void, address: *const c_void,
            buffer: *mut c_void, size: usize, read: *mut usize) -> i32;
        fn RtlCaptureStackBackTrace(skip: u32, count: u32, frames: *mut *mut c_void,
            hash: *mut u32) -> u16;
    }
    pub struct Observer { base: usize, folder: PathBuf, samples: Mutex<BTreeMap<usize, usize>> }
    fn words(address: usize, count: usize) -> Option<Vec<usize>> {
        if address == 0 || count > 128 { return None; }
        let mut data = vec![0usize; count]; let mut read = 0;
        let ok = unsafe { ReadProcessMemory(GetCurrentProcess(), address as *const c_void,
            data.as_mut_ptr().cast(), count * size_of::<usize>(), &mut read) };
        (ok != 0 && read == count * size_of::<usize>()).then_some(data)
    }
    fn rva(base: usize, address: usize) -> Option<String> {
        address.checked_sub(base).filter(|v| *v < 0x58f8000).map(|v| format!("0x{v:x}"))
    }
    impl Observer {
        fn emit(&self, value: Value) {
            if std::fs::create_dir_all(&self.folder).is_err() { return; }
            if let Ok(mut f) = OpenOptions::new().create(true).append(true)
                .open(self.folder.join("observations.jsonl")) {
                if let Ok(line) = serde_json::to_string(&value) { let _ = writeln!(f, "{line}"); }
            }
        }
        unsafe fn sample(&self, raw: *mut SimCtxV1, start: bool) {
            let Some(sim) = StableSim::from_raw(raw) else { return };
            let tick = sim.tick();
            if !start && ![1, 60, 180, 300, 600, 1200].contains(&tick) { return; }
            let key = (*raw).state as usize;
            let Ok(mut samples) = self.samples.lock() else { return };
            if samples.len() >= 64 && !samples.contains_key(&key) { return; }
            let count = samples.entry(key).or_default();
            if start { *count = 0; }
            if *count >= 8 { return; }
            *count += 1; drop(samples);
            if rva(self.base, (*raw).sim as usize).as_deref() != Some("0x4262988") {
                self.emit(json!({"kind":"unsupported-sdk-table","tick":tick,"engineMutations":0}));
                return;
            }
            let state = words(key, 4);
            // Private host prefix is inspected only after exact executable identity admission.
            // This reports an observed receiver; it does not modify or invoke its trait slots.
            let inner_table = state.as_ref().and_then(|s| rva(self.base, s[1]));
            let inner = state.as_ref().and_then(|s| {
                rva(self.base, s[1])?;
                words(s[1], 100).map(|v| v.iter().enumerate().map(|(i, p)| {
                    json!({"offset":format!("0x{:x}", i*8),"targetRva":rva(self.base,*p)})
                }).collect::<Vec<_>>())
            });
            let mut frames = [std::ptr::null_mut(); 32];
            let n = RtlCaptureStackBackTrace(0, 32, frames.as_mut_ptr(), std::ptr::null_mut()) as usize;
            let callers: Vec<_> = frames[..n].iter().filter_map(|v| rva(self.base, *v as usize)).collect();
            let champions: Vec<_> = (0..sim.champion_count()).filter_map(|i| sim.get_entity(sim.champion_id_at(i)))
                .map(|e| json!({"id":e.id(),"team":e.team(),"hp":e.hp(),"pos":e.pos(),
                    "targetable":e.is_targetable(),"visibleToBlue":sim.is_visible(0,e.id()),
                    "visibleToRed":sim.is_visible(1,e.id())})).collect();
            self.emit(json!({"version":1,"kind":"match-observation","exeSha256":EXE_SHA,
                "tick":tick,"seed":sim.seed(),"start":start,"simulationDiagnosticLabel":format!("{key:x}"),
                "sdkTableRva":rva(self.base,(*raw).sim as usize),"internalTableRva":inner_table,
                "internalTableSlots":inner,"gameCallerRvas":callers,"champions":champions,
                "rngConsumed":false,"engineMutations":0,"hookInstalled":false}));
        }
    }
    unsafe extern "C" fn destroy(p: *mut c_void) {
        let _ = catch_unwind(AssertUnwindSafe(|| drop(Box::from_raw(p as *mut Observer))));
    }
    unsafe extern "C" fn start(p: *mut c_void, sim: *mut SimCtxV1) {
        let _ = catch_unwind(AssertUnwindSafe(|| (&*(p as *const Observer)).sample(sim, true)));
    }
    unsafe extern "C" fn tick(p: *mut c_void, sim: *mut SimCtxV1, _: u64) {
        let _ = catch_unwind(AssertUnwindSafe(|| (&*(p as *const Observer)).sample(sim, false)));
    }
    static VTABLE: MatchHookVtableV1 = MatchHookVtableV1 {
        size: size_of::<MatchHookVtableV1>(), destroy: Some(destroy),
        on_match_start: Some(start), on_match_tick: Some(tick), check_match_end: None,
    };
    pub unsafe fn install(export: *mut ModExportV1) {
        let Some(exe) = std::env::current_exe().ok() else { return };
        let Some(parent) = exe.parent() else { return };
        let folder = parent.join("mods/tfm2_realm_observer");
        if !folder.join("realm_probe.flag").is_file() { return; }
        let Ok(bytes) = std::fs::read(&exe) else { return };
        let digest = format!("{:x}", Sha256::digest(&bytes));
        let base = GetModuleHandleW(std::ptr::null()) as usize;
        let observer = Observer { base, folder: folder.join("logs"), samples: Mutex::new(BTreeMap::new()) };
        observer.emit(json!({"version":1,"kind":"observer-load","exeSha256":digest,
            "identityMatches":digest==EXE_SHA,"nativeBackendEnabled":false,"engineMutations":0}));
        if digest != EXE_SHA || base == 0 { return; }
        (*export).match_hook = MatchHookRegV1 {
            userdata: Box::into_raw(Box::new(observer)).cast(), vtable: &VTABLE,
        };
    }
}

#[no_mangle]
pub extern "C" fn tfm2_mod_required_abi_level() -> u32 { 10 }

/// Own-export initialization only; no engine table is overwritten.
/// # Safety
/// `host` must be the host's live size-prefixed stable API table.
#[no_mangle]
pub unsafe extern "C" fn tfm2_mod_entry_stable(host: *const HostApiV1) -> *mut ModExportV1 {
    if host.is_null() { return std::ptr::null_mut(); }
    std::panic::catch_unwind(|| {
        if StableHost::from_raw(host).is_none() { return std::ptr::null_mut(); }
        let export = StableMod::new("tfm2_realm_observer").into_export_raw();
        #[cfg(all(windows, feature = "observe-runtime"))]
        observe::install(export);
        export
    }).unwrap_or(std::ptr::null_mut())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn null_host_is_rejected() {
        assert!(unsafe { tfm2_mod_entry_stable(std::ptr::null()) }.is_null());
    }
    #[test]
    fn no_flag_registers_no_gameplay_or_controller() {
        let mut host: HostApiV1 = unsafe { std::mem::zeroed() };
        host.size = std::mem::size_of::<HostApiV1>(); host.host_abi_level = 10;
        let export = unsafe { tfm2_mod_entry_stable(&host) };
        assert!(!export.is_null());
        unsafe {
            assert_eq!((*export).required_abi_level, 10);
            assert!((*export).match_hook.is_null());
            assert_eq!((*export).champions_len, 0);
            assert_eq!((*export).player_ai_len, 0);
            assert_eq!((*export).native_effects_len, 0);
            (*export).destroy.unwrap()(export);
        }
    }
}
