use std::ops::Add;

use anyhow::{Result, bail};
use log::{debug, info, warn};

use ttd::replay::{DataAccessMask, MemoryWatchpointData, RegisterContext, ReplayEngine};

fn find_export(_replay: &ReplayEngine, _module_name: &str, _export_name: &str) -> Result<u64> {
    Ok(0x0104290)
}

/// For this ONweek demo, we want to extract any shellcode that is encoded
/// or encrypted in the trace. We achieve this by watching VirtualProtect
/// calls with the PAGE_EXECUTE_READWRITE.
/// For each of those calls, we jump ahead to the time the culprit address
/// is being executed, and we print the disassembled payload.
///
fn main() -> Result<()> {
    // region: Preamble boiler plate
    env_logger::Builder::new().filter_level(log::LevelFilter::max()).init();
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        bail!("No trace provided");
    }

    let now = std::time::Instant::now();
    // endregion: Preamble boiler plate

    let replay = ReplayEngine::new()?;
    let trace_path = std::path::Path::new(args[1].as_str());
    replay.load(trace_path)?;
    info!("loaded trace {:?}", &trace_path);

    //
    // 1. Locate kernel32!VirtualProtect
    //
    let kernelbase_base_address = replay.get_module_base_address("kernelbase.dll")?;
    let virtualprotect_address = kernelbase_base_address.add(find_export(&replay, "kernelbase", "VirtualProtect")?);
    debug!("Watching calls to kernelbase!VirtualProtect(RWX) (at {:#x})", virtualprotect_address);

    //
    // 2. Set a (execute) memory breakpoint to kernelbase!virtualprotect
    //
    let watch_point = MemoryWatchpointData {
        Address: virtualprotect_address,
        Size: 1,
        AccessMask: DataAccessMask::Execute.bits(),
        ..Default::default()
    };
    if !replay.add_memory_watchpoint(&watch_point)? {
        bail!("Failed to set watchpoint at {:x}", virtualprotect_address);
    }

    let read_u64 = |addr: u32| -> Result<u32> {
        let mem = replay.read_current_memory(addr.into(), replay.pointer_size()?)?;
        Ok(u32::from_le_bytes(mem.try_into().unwrap()))
    };

    let fmt = zydis::Formatter::intel();
    let dec = zydis::Decoder::new32();

    loop {
        //
        // 3. Play the trace
        //
        let result = replay.replay_forward(None)?;
        if result.stop_reason != ttd::replay::EventType::MemoryWatchpoint {
            break;
        }
        debug!("fwdreplay reason: {}, executed {} insns", result.stop_reason, result.instructions_executed);
        debug!("context at {}:\n{}", &replay.get_position()?, &replay.get_thread_context()?);

        //
        // 4. Inspect the `flNewProtect` argument, looking PAGE_EXECUTE_READWRITE
        //
        const PAGE_EXECUTE_READWRITE: u32 = 0x40;

        let (arg0, arg1, arg2, arg3) = match replay.get_thread_context()? {
            RegisterContext::X86(ctx) => {
                let esp = ctx.Esp;
                (read_u64(esp + 0x04)?, read_u64(esp + 0x08)?, read_u64(esp + 0x0c)?, read_u64(esp + 0x10)?)
            }
            _ => unimplemented!(),
        };

        if arg2 != PAGE_EXECUTE_READWRITE {
            continue;
        }
        debug!("kernelbase!VirtualProtect({arg0:x}, {arg1:x}, {arg2:x}, {arg3:x}) found, setting watch point");

        //
        // 5. Set an execution watchpoint on execution to that address
        // And advance to this address
        //
        replay.remove_memory_watchpoint(&watch_point)?;
        replay.add_memory_watchpoint(&MemoryWatchpointData {
            Address: arg0.into(),
            Size: 1,
            AccessMask: DataAccessMask::Execute.bits(),
            ..Default::default()
        })?;

        replay.replay_forward(None)?;

        let ctx = match replay.get_thread_context()? {
            RegisterContext::X86(ctx) => ctx,
            _ => unimplemented!(),
        };
        let mem = replay.read_current_memory(ctx.Eip.into(), arg1 as usize)?;

        //
        // 6. When the watchpoint is hit, dump all the instructions
        //
        info!("Dumping shellcode at IP={:x} (sz={})", ctx.Eip, arg1);
        let decoder_iter = dec.decode_all::<zydis::VisibleOperands>(&mem, ctx.Eip.into());
        for (idx, insn_info) in decoder_iter.enumerate() {
            let insn_info = insn_info?;
            let ip = insn_info.0;
            let insn = &insn_info.2;
            println!("{:#08x} {}", ip, fmt.format(Some(ip), insn)?);
            if idx >= 20 {
                println!("...");
                warn!("Skipping following instructions...");
                break;
            }
        }

        break;
    }

    debug!("end position {}", &replay.get_position()?);
    debug!("done in {:.4?}", now.elapsed());
    Ok(())
}
