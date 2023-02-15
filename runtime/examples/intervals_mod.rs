// Copyright 2018-2023 the lifegit authors. All rights reserved. MIT license.

use deno_core;
use deno_core::error::AnyError;
use deno_core::{resolve_url, resolve_url_or_path, v8, FsModuleLoader};
use deno_runtime;
use deno_runtime::deno_broadcast_channel::InMemoryBroadcastChannel;
use deno_runtime::deno_web::BlobStore;
use deno_runtime::permissions::PermissionsContainer;
use deno_runtime::worker::MainWorker;
use deno_runtime::worker::WorkerOptions;
use deno_runtime::BootstrapOptions;
use std::rc::Rc;
use std::sync::Arc;

fn get_error_class_name(e: &AnyError) -> &'static str {
  deno_runtime::errors::get_error_class_name(e).unwrap_or("Error")
}

const MOD_TIME: &str = r#"
const getTime = ()=> new Date().getTime();
export default getTime;
"#;

const MOD_FUNC: &str = r#"
import getTime from './time.js'

const t = getTime();
console.log(t);

export default t;
"#;

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

  worker
    .js_runtime
    .load_side_module(
      &resolve_url("file:///time.js").unwrap(),
      Some(MOD_TIME.to_owned()),
    )
    .await
    .expect("TODO: panic message");

  let module_id = worker
    .js_runtime
    .load_side_module(
      &resolve_url("file:///main.js").unwrap(),
      Some(MOD_FUNC.to_owned()),
    )
    .await
    .expect("TODO: panic message");

  worker.js_runtime.mod_evaluate(module_id);
  worker.js_runtime.run_event_loop(false).await.unwrap();

  let module_namespace =
    worker.js_runtime.get_module_namespace(module_id).unwrap();
  let scope = &mut worker.js_runtime.handle_scope();
  let module_namespace = v8::Local::<v8::Object>::new(scope, module_namespace);

  let default_export_name = v8::String::new(scope, "default").unwrap();
  let binding = module_namespace.get(scope, default_export_name.into());

  let value = binding.unwrap();
  println!(
    "value is number ? {}; value = {}",
    value.is_number(),
    value.number_value(scope).unwrap()
  );

  Ok(())
}
