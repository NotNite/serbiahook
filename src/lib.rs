use std::ffi::CString;

use anyhow::Ok;
use retour::static_detour;
use windows::Win32::System::Console::{AttachConsole, ATTACH_PARENT_PROCESS};

type FnImGuiText = unsafe extern "C" fn(*const i8);

static_detour! {
    static ImGuiText: unsafe extern "C" fn(*const i8);
}

unsafe fn fix_jmp_call(ptr: *mut u8) -> *mut u8 {
    let offset = (ptr.wrapping_add(1)) as *mut i32;
    let new_addr = ptr.wrapping_add(5).wrapping_add(*offset as usize);
    new_addr as *mut u8
}

unsafe fn fix_stdout() -> anyhow::Result<()> {
    AttachConsole(ATTACH_PARENT_PROCESS);
    Ok(())
}

unsafe fn patch_imgui_text() -> anyhow::Result<()> {
    let sig = skidscan::signature!("E8 ?? ?? ?? ?? EB 5A");
    let addr = sig
        .scan_module("scenesystem.dll")
        .expect("Failed to find signature");
    let addr = fix_jmp_call(addr);
    let func = std::mem::transmute::<_, FnImGuiText>(addr);

    ImGuiText.initialize(func, |_| {
        let cstr = CString::new("SerbiaHook on top").unwrap();
        ImGuiText.call(cstr.as_ptr());
    })?;
    ImGuiText.enable()?;

    Ok(())
}

unsafe fn real_ctor() -> anyhow::Result<()> {
    fix_stdout()?;
    println!("hello from SerbiaHook");

    patch_imgui_text()?;

    println!("SerbiaHook loaded");
    Ok(())
}

#[ctor::ctor]
fn ctor() {
    unsafe {
        if let Err(e) = real_ctor() {
            eprintln!("SerbiaHook: {}", e);
        }
    }
}
