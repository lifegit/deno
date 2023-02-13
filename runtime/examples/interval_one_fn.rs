// Copyright 2018-2023 the lifegit authors. All rights reserved. MIT license.

use std::collections::BTreeMap;
use deno_core::error::{AnyError};
use deno_core::v8::{HandleScope, Local};
use deno_core::{resolve_url_or_path, serde_json, v8, FsModuleLoader};
use deno_runtime;
use deno_runtime::common::{Entry};
use deno_runtime::deno_broadcast_channel::InMemoryBroadcastChannel;
use deno_runtime::deno_web::BlobStore;
use deno_runtime::permissions::PermissionsContainer;
use deno_runtime::types::TypedValue;
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

  let mut scope = worker.js_runtime.handle_scope();
  let mut func =
    RuntimeEngine::new("function f1(n) { return n+1 }", "f1", &mut scope);
  let mut num: f64 = 0.0;
  let mut interval = time::interval(time::Duration::from_secs(2));
  loop {
    // parameters
    let mut entry = Entry::default();
    let val = TypedValue::Number(num);
    entry.set_data_type(val.get_type());
    entry.value = val.get_data();

    // run
    let result = func
      .call_fn(&TypedValue::from(&entry))
      .unwrap_or(TypedValue::Invalid);

    // result
    println!("res = {}", result.to_string());
    assert_eq!(result.to_string(), (num + 1.0).to_string());
    interval.tick().await;
    num = num + 1.0;
  }
}

pub struct RuntimeEngine<'s, 'i> {
  context_scope: v8::ContextScope<'i, v8::HandleScope<'s>>,
  ctx: Local<'s, v8::Context>,
  process_fn: Option<Local<'s, v8::Function>>,
}
impl<'s, 'i> RuntimeEngine<'s, 'i>
where
  's: 'i,
{
  pub fn new(
    source_code: &str,
    fn_name: &str,
    isolated_scope: &'i mut v8::HandleScope<'s, ()>,
  ) -> Self {
    let ctx = v8::Context::new(isolated_scope);
    let mut scope = v8::ContextScope::new(isolated_scope, ctx);
    let code = v8::String::new(&mut scope, source_code).unwrap();

    let script = v8::Script::compile(&mut scope, code, None).unwrap();

    let mut self_ = RuntimeEngine {
      context_scope: scope,
      ctx,
      process_fn: None,
    };

    self_.execute_script(script);

    let process_str = v8::String::new(&mut self_.context_scope, fn_name);

    let fn_value = ctx
      .global(&mut self_.context_scope)
      .get(&mut self_.context_scope, process_str.unwrap().into())
      .unwrap();
    let fn_opt = v8::Local::<v8::Function>::try_from(fn_value);
    let process_fn = if fn_opt.is_ok() {
      Some(fn_opt.unwrap())
    } else {
      None
    };
    self_.process_fn = process_fn;
    self_
  }

  fn execute_script(&mut self, script: Local<v8::Script>) {
    let handle_scope = &mut v8::HandleScope::new(&mut self.context_scope);
    let try_catch = &mut v8::TryCatch::new(handle_scope);

    if script.run(try_catch).is_none() {
      try_catch_log(try_catch);
    }
  }

  /// call with one argument
  pub fn call_fn(&mut self, val: &TypedValue) -> Option<TypedValue> {
    let scope = &mut v8::HandleScope::new(&mut self.context_scope);
    let ref mut try_catch = v8::TryCatch::new(scope);
    let global = self.ctx.global(try_catch).into();
    let arg_vals = [wrap_value(val, try_catch)];

    let process_fn = self.process_fn.as_ref().unwrap();

    match process_fn.call(try_catch, global, &arg_vals) {
      Some(v) => to_typed_value(v, try_catch),
      None => {
        try_catch_log(try_catch);
        None
      }
    }
  }
}

/*
wrap_value() will convert a TypedValue into v8::Value.
 */
pub fn wrap_value<'s, 'i, 'p>(
  typed_val: &'p TypedValue,
  scope: &'i mut HandleScope<'s, ()>,
) -> v8::Local<'s, v8::Value>
where
  's: 'i,
{
  match typed_val {
    TypedValue::String(value) => {
      let v8_str = v8::String::new(scope, value.as_str()).unwrap();
      v8::Local::<v8::Value>::from(v8_str)
    }
    TypedValue::BigInt(value) => {
      let ctx = v8::Context::new(scope);
      let context_scope = &mut v8::ContextScope::new(scope, ctx);

      let v8_i64 = v8::BigInt::new_from_i64(context_scope, *value);
      v8::Local::<v8::Value>::from(v8_i64)
    }
    TypedValue::Boolean(value) => {
      let v8_bool = v8::Boolean::new(scope, *value);
      v8::Local::<v8::Value>::from(v8_bool)
    }
    TypedValue::Number(value) => {
      let v8_number = v8::Number::new(scope, *value);
      v8::Local::<v8::Value>::from(v8_number)
    }
    TypedValue::Null => {
      let v8_null = v8::null(scope);
      v8::Local::<v8::Value>::from(v8_null)
    }
    TypedValue::Object(value) => {
      let ctx = v8::Context::new(scope);
      let context_scope = &mut v8::ContextScope::new(scope, ctx);

      let v8_obj = if value.is_empty() {
        v8::Object::new(context_scope)
      } else {
        let v8_obj = v8::Object::new(context_scope);

        value.iter().for_each(|(key, value)| {
          let key = v8::String::new(context_scope, key.as_str()).unwrap();

          let value = wrap_value(value, context_scope);
          v8_obj.set(context_scope, key.into(), value);
        });
        v8_obj
      };

      v8::Local::<v8::Value>::from(v8_obj)
    }
    TypedValue::Invalid => {
      let v8_undefined = v8::undefined(scope);
      v8::Local::<v8::Value>::from(v8_undefined)
    }
    TypedValue::Array(_) => {
      let val = serde_json::to_string(&typed_val.to_json_value()).unwrap();
      let val = v8::String::new(scope, &val).unwrap();

      let ctx = v8::Context::new(scope);
      let context_scope = &mut v8::ContextScope::new(scope, ctx);

      match v8::json::parse(context_scope, v8::Local::<v8::String>::from(val)) {
        Some(local) => local,
        None => v8::Local::from(v8::undefined(context_scope)),
      }
    }
  }
}

/*
to_typed_value() will convert v8::Value into TypedValue.
 */
pub fn to_typed_value<'s>(
  local: v8::Local<v8::Value>,
  handle_scope: &'s mut v8::HandleScope,
) -> Option<TypedValue> {
  if local.is_undefined() {
    return Some(TypedValue::Invalid);
  }
  if local.is_big_int() {
    return local
      .to_big_int(handle_scope)
      .filter(|val| {
        let (_, ok) = val.i64_value();
        ok
      })
      .map(|val| {
        let (v, _) = val.i64_value();
        TypedValue::BigInt(v)
      });
  }
  if local.is_number() {
    return local
      .number_value(handle_scope)
      .map(|val| TypedValue::Number(val));
  }

  if local.is_null() {
    return Some(TypedValue::Null);
  }
  if local.is_boolean() {
    return Some(TypedValue::Boolean(local.is_true()));
  }
  if local.is_string() {
    return local
      .to_string(handle_scope)
      .map(|val| TypedValue::String(val.to_rust_string_lossy(handle_scope)));
  }

  if local.is_array() {
    return local.to_object(handle_scope).map(|obj| {
      let mut arr = vec![];
      let mut index = 0;
      loop {
        let has_index_opt = obj.has_index(handle_scope, index);
        if has_index_opt.is_some() && !has_index_opt.unwrap() {
          break;
        }
        if has_index_opt.is_none() {
          break;
        }

        let value_opts = obj.get_index(handle_scope, index);
        if value_opts.is_none() {
          break;
        }

        let val = value_opts.unwrap();
        arr.push(to_typed_value(val, handle_scope).unwrap_or_default());
        index = index + 1;
      }

      TypedValue::Array(arr)
    });
  }

  if local.is_object() {
    let args = v8::GetPropertyNamesArgsBuilder::default().build();
    return local.to_object(handle_scope).and_then(|obj| {
      obj.get_own_property_names(handle_scope, args).map(|names| {
        let mut map = BTreeMap::default();
        let arr = &*names;
        for index in 0..arr.length() {
          arr.get_index(handle_scope, index).iter().for_each(|key| {
            let value = obj.get(handle_scope, key.clone()).unwrap();
            let v = to_typed_value(value, handle_scope).unwrap();
            map.insert(key.to_rust_string_lossy(handle_scope), v);
          })
        }
        TypedValue::Object(map)
      })
    });
  }

  Some(TypedValue::Invalid)
}
fn try_catch_log(try_catch: &mut v8::TryCatch<v8::HandleScope>) {
  let exception = try_catch.exception().unwrap();
  let exception_string = exception
    .to_string(try_catch)
    .unwrap()
    .to_rust_string_lossy(try_catch);
  tracing::error!("{}", exception_string);
}
