use wasmedge_sdk::ValType;
use wasmedge_sys::WasmValue;

#[derive(Debug, Clone)]
pub enum WasmVal {
    I32(i32),
    I64(i64),
    F32(f32),
    F64(f64),
    V128(i128),
    None,
}

impl PartialEq for WasmVal {
    fn eq(&self, other: &Self) -> bool {
        use WasmVal::*;
        match (self, other) {
            (I32(i), I32(other)) => *i == *other,
            (I64(i), I64(other)) => *i == *other,
            (F32(i), F32(other)) => *i == *other,
            (F64(i), F64(other)) => *i == *other,
            (V128(i), V128(other)) => *i == *other,
            (None, None) => true,
            _ => false,
        }
    }
}
impl Eq for WasmVal {}

impl From<WasmValue> for WasmVal {
    fn from(raw_val: WasmValue) -> Self {
        match raw_val.ty() {
            ValType::I32 => WasmVal::I32(raw_val.to_i32()),
            ValType::I64 => WasmVal::I64(raw_val.to_i64()),
            ValType::F32 => WasmVal::F32(raw_val.to_f32()),
            ValType::F64 => WasmVal::F64(raw_val.to_f64()),
            ValType::V128 => WasmVal::V128(raw_val.to_v128()),
            _ => WasmVal::None,
        }
    }
}
