//! Native code for tapes: Cranelift on native targets; none on wasm32 (tapes stay interpreted there).
#[cfg(not(target_arch = "wasm32"))]
#[path = "jit_native.rs"]
mod native;
#[cfg(not(target_arch = "wasm32"))]
pub use native::*;

#[cfg(target_arch = "wasm32")]
pub struct Jit { pub f: unsafe extern "C" fn(*mut f64) }
#[cfg(target_arch = "wasm32")]
pub fn compile(_ins: &[(u32, crate::expr::Op)], _outs: &[u32], _nregs: usize) -> Option<Jit> { None }
