use wasmtime::component::Resource;

use crate::durable_host::DurableWorkerCtx;
use crate::metrics::wasm::record_host_function_call;
use crate::workerctx::WorkerCtx;
use wasmtime_wasi::preview2::bindings::wasi::cli::stdin::{Host, InputStream};

impl<Ctx: WorkerCtx> Host for DurableWorkerCtx<Ctx> {
    fn get_stdin(&mut self) -> anyhow::Result<Resource<InputStream>> {
        record_host_function_call("cli::stdin", "get_stdin");
        self.as_wasi_view().get_stdin()
    }
}