#![allow(non_camel_case_types)]
#![allow(unused_unsafe)]
use crate::ctx::WasiCtx;
use crate::fdentry::{determine_type_rights, FdEntry};
use crate::memory::*;
use crate::{host, wasm32};

use cast::From as _0;
use lucet_runtime::vmctx::Vmctx;
use lucet_runtime::{lucet_hostcall_terminate, lucet_hostcalls};
use std::str;

use nix::convert_ioctl_res;
use nix::libc::c_int;
use std::ffi::{OsStr, OsString};
use std::os::unix::prelude::{FromRawFd, OsStrExt, OsStringExt, RawFd};
use std::time::{SystemTime, UNIX_EPOCH};
use std::{cmp, slice};

use wasabi::{js, WASABI};

fn load_value(vmctx: &mut Vmctx, v_addr: wasm32::int32_t) -> Option<(i64, bool)> {
    match unsafe { dec_slice_of::<u8>(vmctx, v_addr as u32, 8) } {
        Ok((ptr, len)) => Some(js::load_value(unsafe {
            &std::slice::from_raw_parts(ptr, len)
        })),
        Err(e) => None,
    }
}

fn store_bool(vmctx: &mut Vmctx, addr: wasm32::int32_t, v: bool) {
    let buf = match unsafe { dec_slice_of::<u8>(vmctx, addr as u32, 1) } {
        Ok((ptr, len)) => unsafe { std::slice::from_raw_parts_mut(ptr, len) },
        Err(e) => return,
    };
    buf.copy_from_slice(&[{
        if v {
            1
        } else {
            0
        }
    }]);
}
fn store_value(vmctx: &mut Vmctx, addr: wasm32::int32_t, v: (i64, bool)) {
    let b = js::store_value(v);
    let buf = match unsafe { dec_slice_of::<u8>(vmctx, addr as u32, 8) } {
        Ok((ptr, len)) => unsafe { std::slice::from_raw_parts_mut(ptr, len) },
        Err(e) => return,
    };
    buf.copy_from_slice(&b);
}

lucet_hostcalls! {

    #[no_mangle]
    pub unsafe extern "C" fn __tinygo_io_get_stdout(
        &mut _vmctx,
    ) -> wasm32::uint32_t {
        1
    }

    #[no_mangle]
    pub unsafe extern "C" fn __tinygo_tick(
        &mut _vmctx,
    ) -> f64 {
        let start = SystemTime::now();
        let since_the_epoch = start.duration_since(UNIX_EPOCH).unwrap();
        let ns = (since_the_epoch.as_secs() * 1_000_000_000 + since_the_epoch.subsec_nanos() as u64) as f64;
        ns/1_000_000.0
    }

    #[no_mangle]
    pub unsafe extern "C" fn __tinygo_sleepticks(
        &mut _vmctx,
        timeout: f64,
    ) -> () {
        let mut wasabi = WASABI.lock().unwrap();
        wasabi.timeout_heap.add_float(timeout);
        println!("__tinygo_sleepticks {:?}", (timeout));
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
        &mut vmctx,
        retval: wasm32::int32_t,
        v_addr: wasm32::int32_t,
        p_ptr: wasm32::int32_t,
        p_len: wasm32::int32_t,
    ) ->  () {
        let key = match unsafe {dec_slice_of::<u8>(vmctx, p_ptr as u32, p_len as u32) } {
            Ok((ptr, len)) => str::from_utf8(std::slice::from_raw_parts(ptr, len)).unwrap(),
            Err(e) => return,
        };

        let wasabi = WASABI.lock().unwrap();
        let v = match wasabi.js.static_strings.get(&key) {
            Some(v) => v,
            None => return,
        };

        let result = wasabi.js.reflect_get(load_value(vmctx, v_addr).unwrap().0, v).unwrap();
        println!("__js_value_get {:?}", (result, v));

        store_value(vmctx, retval, result)
    }


    #[no_mangle]
    pub unsafe extern "C" fn __js_value_new(
        &mut vmctx,
       ret_addr: wasm32::int32_t,
       v_addr: wasm32::int32_t,
       args_ptr: wasm32::int32_t,
       args_len: wasm32::int32_t,
       args_cap: wasm32::int32_t,
    ) ->  () {
        let mut wasabi = WASABI.lock().unwrap();
        let v = load_value(vmctx, v_addr).unwrap();

        let mut args = Vec::new();
        for n in 0..args_len {
            args.push(load_value(vmctx, args_ptr + n * 8).unwrap())
        }
        println!("__js_value_new {:?}", (&args, v, args_len));

        let result = wasabi.js.reflect_construct(v.0, args).unwrap();
        println!("{:?}", result);

        store_value(vmctx, ret_addr, result);
        store_bool(vmctx, ret_addr+8, true);
    }

    #[no_mangle]
    pub unsafe extern "C" fn __js_value_call(
        &mut vmctx,
        ret_addr: wasm32::int32_t,
        v_addr: wasm32::int32_t,
        m_ptr: wasm32::int32_t,
        m_len: wasm32::int32_t,
        args_ptr: wasm32::int32_t,
        args_len: wasm32::int32_t,
        args_cap: wasm32::int32_t,
    ) ->  () {
        let mut wasabi = WASABI.lock().unwrap();
        let v = load_value(vmctx, v_addr).unwrap();


        let m_str = match unsafe {dec_slice_of::<u8>(vmctx, m_ptr as u32, m_len as u32) } {
            Ok((ptr, len)) => str::from_utf8(std::slice::from_raw_parts(ptr, len)).unwrap(),
            Err(e) => return,
        };
        let property_key = match wasabi.js.static_strings.get(&m_str) {
            Some(v) => v,
            None => return,
        };

        let m = wasabi.js.reflect_get(v.0, property_key).unwrap();


        let mut args = Vec::new();
        for n in 0..args_len {
            args.push(load_value(vmctx, args_ptr + n * 8).unwrap())
        }

        let target = m.0;
        let this_argument = v.0;
        let target_name = wasabi.js.get_object_name(target).unwrap();
        let this_argument_name = wasabi.js.get_object_name(this_argument).unwrap();
        let result =  match (target_name, this_argument_name) {
            ("getTimezoneOffset", "Date") => Some((0, false)),
            _ => return,
        }.unwrap();
        store_value(vmctx, ret_addr, result);
        store_bool(vmctx, ret_addr+8, true);
        println!("__js_value_call {:?}", (&args, v, args_len, m));
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
        std::ptr::read_volatile(__tinygo_io_get_stdout as *const extern "C" fn());
    }
}
