// Copyright 2018-2023 the lifegit authors. All rights reserved. MIT license.

use deno_core::error::AnyError;
use deno_core::{resolve_url_or_path, FsModuleLoader};
use deno_runtime::deno_broadcast_channel::InMemoryBroadcastChannel;
use deno_runtime::deno_web::BlobStore;
use deno_runtime::permissions::PermissionsContainer;
use deno_runtime::worker::MainWorker;
use deno_runtime::worker::WorkerOptions;
use deno_runtime::BootstrapOptions;
use std::rc::Rc;
use std::sync::Arc;
use tokio::time;

fn get_error_class_name(e: &AnyError) -> &'static str {
  deno_runtime::errors::get_error_class_name(e).unwrap_or("Error")
}

#[tokio::main]
async fn main() -> Result<(), AnyError> {
  let module_loader = Rc::new(FsModuleLoader);
  let create_web_worker_cb = Arc::new(|_| {
    todo!("Web workers are not supported in the example");
  });
  let web_worker_event_cb = Arc::new(|_| {
    todo!("Web workers are not supported in the example");
  });

  let options = WorkerOptions {
    bootstrap: BootstrapOptions {
      args: vec![],
      cpu_count: 1,
      debug_flag: false,
      enable_testing_features: false,
      locale: deno_core::v8::icu::get_language_tag(),
      location: None,
      no_color: false,
      is_tty: false,
      runtime_version: "x".to_string(),
      ts_version: "x".to_string(),
      unstable: false,
      user_agent: "hello_runtime".to_string(),
      inspect: false,
    },
    extensions: vec![],
    extensions_with_js: vec![],
    startup_snapshot: None,
    unsafely_ignore_certificate_errors: None,
    root_cert_store: None,
    seed: None,
    source_map_getter: None,
    format_js_error_fn: None,
    web_worker_preload_module_cb: web_worker_event_cb.clone(),
    web_worker_pre_execute_module_cb: web_worker_event_cb,
    create_web_worker_cb,
    maybe_inspector_server: None,
    should_break_on_first_statement: false,
    should_wait_for_inspector_session: false,
    module_loader,
    npm_resolver: None,
    get_error_class_fn: Some(&get_error_class_name),
    cache_storage_dir: None,
    origin_storage_dir: None,
    blob_store: BlobStore::default(),
    broadcast_channel: InMemoryBroadcastChannel::default(),
    shared_array_buffer_store: None,
    compiled_wasm_module_store: None,
    stdio: Default::default(),
  };
  let mut worker = MainWorker::bootstrap_from_options(
    resolve_url_or_path("./$deno$stdin.ts").unwrap(),
    PermissionsContainer::allow_all(),
    options,
  );

  let mut interval = time::interval(time::Duration::from_secs(2));
  loop {
    let global = worker
      .js_runtime
      .execute_script(
        "a.js",
        "function f(a) { return new Date().getTime() }; f();",
      )
      .unwrap();
    let scope = &mut worker.js_runtime.handle_scope();
    let value = global.open(scope).integer_value(scope).unwrap(); //  value.to_string(scope).unwrap().to_rust_string_lossy(scope),
    println!("res = {}", value);

    interval.tick().await;
  }
}
