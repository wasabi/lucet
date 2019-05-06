#![allow(non_camel_case_types)]
#![allow(unused_unsafe)]
use crate::ctx::WasiCtx;
use crate::fdentry::{determine_type_rights, FdEntry};
use crate::memory::*;
use crate::{host, wasm32};

use cast::From as _0;
use lucet_runtime::vmctx::Vmctx;
use lucet_runtime::{lucet_hostcall_terminate, lucet_hostcalls};

use nix::convert_ioctl_res;
use nix::libc::c_int;
use std::ffi::{OsStr, OsString};
use std::os::unix::prelude::{FromRawFd, OsStrExt, OsStringExt, RawFd};
use std::time::SystemTime;
use std::{cmp, slice};

lucet_hostcalls! {

    #[no_mangle]
    pub unsafe extern "C" fn __io_get_stdout(
        &mut _vmctx,
    ) -> wasm32::uint32_t {
        1
    }

    #[no_mangle]
    pub unsafe extern "C" fn __tick(
        &mut _vmctx,
    ) -> f64 {
        1.0
    }

    #[no_mangle]
    pub unsafe extern "C" fn _random(
        &mut vmctx,
        buf_ptr: wasm32::uintptr_t,
        buf_len: wasm32::size_t,
        buf_cap: wasm32::size_t,
    ) -> () {
        use rand::{thread_rng, RngCore};

        let buf_len = dec_usize(buf_len);
        let buf_ptr = match unsafe { dec_ptr(vmctx, buf_ptr, buf_len) } {
            Ok(ptr) => ptr,
            Err(e) => return,
        };

        let buf = unsafe { std::slice::from_raw_parts_mut(buf_ptr, buf_len) };

        thread_rng().fill_bytes(buf);
    }


    #[no_mangle]
    pub unsafe extern "C" fn __js_string_val(
        &mut _vmctx,
        ret_ptr: wasm32::int32_t,
        value_ptr: wasm32::int32_t,
        value_len: wasm32::int32_t,
    ) -> () {
        println!("__js_string_val");
    }

    #[no_mangle]
    pub unsafe extern "C" fn __js_value_get(
        &mut _vmctx,
        retval: wasm32::int32_t,
        v_addr: wasm32::int32_t,
        p_ptr: wasm32::int32_t,
        p_len: wasm32::int32_t,
    ) ->  () {
        println!("__js_value_get");
    }


    #[no_mangle]
    pub unsafe extern "C" fn __js_value_load_string(
        &mut _vmctx,
        v_addr: wasm32::int32_t,
        slice_ptr: wasm32::int32_t,
        slice_len: wasm32::int32_t,
        slice_cap: wasm32::int32_t,
    ) ->  () {
        println!("__js_value_load_string");
    }

    #[no_mangle]
    pub unsafe extern "C" fn __js_value_prepare_string(
        &mut _vmctx,
        ret_addr: wasm32::int32_t,
        v_addr: wasm32::int32_t,
    ) ->  () {
        println!("__js_value_prepare_string");
    }

    #[no_mangle]
    pub unsafe extern "C" fn __js_value_call(
        &mut _vmctx,
        ret_addr: wasm32::int32_t,
        v_addr: wasm32::int32_t,
        m_ptr: wasm32::int32_t,
        m_len: wasm32::int32_t,
        args_ptr: wasm32::int32_t,
        args_len: wasm32::int32_t,
        args_cap: wasm32::int32_t,
    ) ->  () {
        println!("__js_value_call");
    }


    #[no_mangle]
    pub unsafe extern "C" fn __js_value_new(
        &mut _vmctx,
       ret_addr: wasm32::int32_t,
       v_addr: wasm32::int32_t,
       args_ptr: wasm32::int32_t,
       args_len: wasm32::int32_t,
       args_cap: wasm32::int32_t,
    ) ->  () {
        println!("__js_value_prepare_string");
    }

    #[no_mangle]
    pub unsafe extern "C" fn __debug(
        &mut _vmctx,
        o: wasm32::int32_t,
        t: wasm32::int32_t,
        f: wasm32::int32_t,
        s: wasm32::int32_t,
    ) ->  () {
        println!("{:?}", (o,t,f,s));
    }


    #[no_mangle]
    pub unsafe extern "C" fn __resource_write(
        &mut vmctx,
        fd: wasm32::int32_t,
        ptr: wasm32::int32_t,
        len: wasm32::int32_t,
    ) -> wasm32::uint32_t {

        let string = match unsafe { dec_slice_of::<u8>(vmctx, ptr as u32, len as u32) } {
            Ok((ptr, len)) => OsStr::from_bytes(unsafe { std::slice::from_raw_parts(ptr, len) }),
            Err(e) => return 0,
        };
        print!("{}", string.to_str().unwrap());
        string.len() as u32
    }

}

#[doc(hidden)]
pub fn ensure_linked() {
    unsafe {
        std::ptr::read_volatile(__io_get_stdout as *const extern "C" fn());
    }
}
