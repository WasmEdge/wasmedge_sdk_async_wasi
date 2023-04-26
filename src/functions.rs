use sys::{CallingFrame, Function, WasmValue};
use wasmedge_sdk::error::HostFuncError;
use wasmedge_sdk::WasmEdgeResult;
use wasmedge_sys as sys;
use wasmedge_sys::ffi;

use std::ffi::c_void;
use std::pin::Pin;

pub type HostFn<T> =
    fn(&mut CallingFrame, &mut T, Vec<WasmValue>) -> Result<Vec<WasmValue>, HostFuncError>;

pub type AsyncHostFn<T> =
    fn(
        CallingFrame,
        &'static mut T,
        Vec<WasmValue>,
    ) -> Box<dyn std::future::Future<Output = Result<Vec<WasmValue>, HostFuncError>> + Send>;

unsafe extern "C" fn wrap_fn<T: 'static>(
    key_ptr: *mut c_void,
    data: *mut std::os::raw::c_void,
    call_frame_ctx: *const ffi::WasmEdge_CallingFrameContext,
    params: *const ffi::WasmEdge_Value,
    param_len: u32,
    returns: *mut ffi::WasmEdge_Value,
    return_len: u32,
) -> ffi::WasmEdge_Result {
    let mut frame = CallingFrame::from_raw(call_frame_ctx as *mut _);
    let data = (data as *mut T).as_mut();
    debug_assert!(data.is_some());
    let data = data.unwrap();

    let real_fn: HostFn<T> = std::mem::transmute(key_ptr);

    let input = {
        let raw_input = unsafe {
            std::slice::from_raw_parts(
                params,
                param_len
                    .try_into()
                    .expect("len of params should not greater than usize"),
            )
        };
        raw_input.iter().map(|r| (*r).into()).collect::<Vec<_>>()
    };

    let return_len = return_len
        .try_into()
        .expect("len of returns should not greater than usize");
    let raw_returns = unsafe { std::slice::from_raw_parts_mut(returns, return_len) };

    match real_fn(&mut frame, data, input) {
        Ok(returns) => {
            assert!(returns.len() == return_len, "[wasmedge-sys] check the number of returns of host function. Expected: {}, actual: {}", return_len, returns.len());
            for (idx, wasm_value) in returns.into_iter().enumerate() {
                raw_returns[idx] = wasm_value.as_raw();
            }
            ffi::WasmEdge_Result { Code: 0 }
        }
        Err(err) => match err {
            HostFuncError::User(code) => unsafe {
                ffi::WasmEdge_ResultGen(ffi::WasmEdge_ErrCategory_UserLevelError, code)
            },
            HostFuncError::Runtime(code) => unsafe {
                ffi::WasmEdge_ResultGen(ffi::WasmEdge_ErrCategory_WASM, code)
            },
        },
    }
}

unsafe extern "C" fn async_wrap_fn<T: 'static>(
    key_ptr: *mut c_void,
    data: *mut std::os::raw::c_void,
    call_frame_ctx: *const ffi::WasmEdge_CallingFrameContext,
    params: *const ffi::WasmEdge_Value,
    param_len: u32,
    returns: *mut ffi::WasmEdge_Value,
    return_len: u32,
) -> ffi::WasmEdge_Result {
    let frame = CallingFrame::from_raw(call_frame_ctx as *mut _);
    let data: &'static mut T = &mut *(data as *mut T);

    let real_fn: AsyncHostFn<T> = std::mem::transmute(key_ptr);

    let args = {
        let raw_input = unsafe {
            std::slice::from_raw_parts(
                params,
                param_len
                    .try_into()
                    .expect("len of params should not greater than usize"),
            )
        };
        raw_input.iter().map(|r| (*r).into()).collect::<Vec<_>>()
    };

    let return_len = return_len
        .try_into()
        .expect("len of returns should not greater than usize");
    let raw_returns = unsafe { std::slice::from_raw_parts_mut(returns, return_len) };

    let r = {
        let async_cx = sys::r#async::AsyncCx::new();
        let mut future = Pin::from(real_fn(frame, data, args));
        match unsafe { async_cx.block_on(future.as_mut()) } {
            Ok(Ok(ret)) => Ok(ret),
            Ok(Err(err)) => Err(err),
            Err(_err) => Err(HostFuncError::User(0x87)),
        }
    };
    match r {
        Ok(returns) => {
            assert!(returns.len() == return_len, "[wasmedge-sys] check the number of returns of host function. Expected: {}, actual: {}", return_len, returns.len());
            for (idx, wasm_value) in returns.into_iter().enumerate() {
                raw_returns[idx] = wasm_value.as_raw();
            }
            ffi::WasmEdge_Result { Code: 0 }
        }
        Err(err) => match err {
            HostFuncError::User(code) => unsafe {
                ffi::WasmEdge_ResultGen(ffi::WasmEdge_ErrCategory_UserLevelError, code)
            },
            HostFuncError::Runtime(code) => unsafe {
                ffi::WasmEdge_ResultGen(ffi::WasmEdge_ErrCategory_WASM, code)
            },
        },
    }
}

pub unsafe fn new_sync_function<T: 'static>(
    ty: &sys::FuncType,
    real_fn: HostFn<T>,
    data: &mut T,
    cost: u64,
) -> WasmEdgeResult<Function> {
    Function::create_with_custom_wrapper(
        ty,
        wrap_fn::<T>,
        real_fn as *mut _,
        data as *mut T as *mut _,
        cost,
    )
}

pub unsafe fn new_async_function<T: 'static>(
    ty: &sys::FuncType,
    real_fn: AsyncHostFn<T>,
    data: &mut T,
    cost: u64,
) -> WasmEdgeResult<Function> {
    Function::create_with_custom_wrapper(
        ty,
        async_wrap_fn::<T>,
        real_fn as *mut _,
        data as *mut T as *mut _,
        cost,
    )
}
