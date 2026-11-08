//! Node-API surface for `@wae/open-desktop`.
//!
//! Every platform difference belongs to the `wae` facade: this crate has no
//! `cfg` branches and no platform-specific messages. Capability facts come from
//! `open_desktop_core::capabilities()` only, and every exported entry point is
//! recorded through `wae.audit/v0`.

#![deny(missing_docs)]

use std::{env, path::PathBuf};

use open_desktop_core::{Operation, PlatformError, capabilities as core_capabilities, version as core_version};
use serde_json::{Value, json};
use wae::{CallContext, Error, JsValue, Result, function, optional_string_prop, register_module};

mod audit;
mod gui;

use audit::Summary;

const APP_TITLE: &str = "OpenDesktop";
const GUI_DIR_ENV: &str = "OPEN_DESKTOP_GUI";

fn actor_kind(value: Option<&str>) -> wae::audit::ActorKind {
    match value {
        Some(value) if value.eq_ignore_ascii_case("agent") => wae::audit::ActorKind::Agent,
        _ => wae::audit::ActorKind::User,
    }
}

fn json_to_js(env: wae::Env, value: Value) -> Result<JsValue> {
    match value {
        Value::Null => env.null(),
        Value::Bool(value) => env.boolean(value),
        Value::Number(number) => match number.as_i64() {
            Some(value) => env.int64(value),
            None => env.double(number.as_f64().unwrap_or(0.0)),
        },
        Value::String(text) => env.string(&text),
        Value::Array(items) => {
            let array = env.array_with_length(items.len())?;
            for (index, item) in items.into_iter().enumerate() {
                array.set_element(index as u32, json_to_js(env, item)?)?;
            }
            Ok(array)
        }
        Value::Object(map) => {
            let object = env.object()?;
            for (key, item) in map {
                object.set_named(&key, json_to_js(env, item)?)?;
            }
            Ok(object)
        }
    }
}

fn value_to_js<T: serde::Serialize>(ctx: &CallContext, value: &T) -> Result<JsValue> {
    let json = serde_json::to_value(value).map_err(|error| Error::new(format!("serialize failed: {error}")))?;
    json_to_js(ctx.env(), json)
}

fn payload(error: &PlatformError) -> Value {
    let mut payload = json!({ "code": error.code(), "capability": error.capability(), "message": error.to_string() });
    if let Some(reason) = error.reason() {
        payload["reason"] = serde_json::to_value(reason).unwrap_or(Value::Null);
    }
    payload
}

fn core_error(error: PlatformError) -> Error {
    match serde_json::to_string(&payload(&error)) {
        Ok(json) => Error::new(format!("wae.error:{json}")),
        Err(_) => Error::new("wae.error:{\"code\":\"failed\",\"message\":\"error payload serialization failed\"}"),
    }
}

fn object_arg(ctx: &CallContext) -> Result<Option<JsValue>> {
    let Some(arg) = ctx.args().first().copied()
    else {
        return Ok(None);
    };
    if arg.is_undefined()? {
        return Ok(None);
    }
    Ok(Some(arg))
}

fn optional_int(obj: Option<JsValue>, name: &str) -> Result<Option<i64>> {
    let Some(obj) = obj
    else {
        return Ok(None);
    };
    let value = obj.get_named(name)?;
    if value.is_undefined()? {
        return Ok(None);
    }
    Ok(Some(value.as_i64()?))
}

fn resolve_gui_dir(requested: Option<String>) -> Result<String> {
    if let Some(dir) = requested
        && !dir.trim().is_empty()
    {
        return Ok(dir);
    }
    if let Ok(dir) = env::var(GUI_DIR_ENV)
        && !dir.trim().is_empty()
    {
        return Ok(dir);
    }
    let candidates = [PathBuf::from("packages/open-desktop/dist/gui"), PathBuf::from("dist/gui"), PathBuf::from("gui")];
    for candidate in candidates {
        if candidate.join("index.html").is_file() {
            return Ok(candidate.display().to_string());
        }
    }
    Err(Error::new("gui content dir not found: pass `contentDir` or build `packages/open-desktop/dist/gui`"))
}

fn query_from_arg(arg: Option<JsValue>) -> Result<wae::audit::Query> {
    let mut query = wae::audit::Query::default();
    let Some(arg) = arg
    else {
        return Ok(query);
    };
    query.actor_kind = optional_string_prop(arg, "kind")?.map(|value| actor_kind(Some(value.as_str())));
    query.actor_id = optional_string_prop(arg, "id")?;
    query.capability = optional_string_prop(arg, "capability")?;
    query.from_ms = optional_int(Some(arg), "fromMs")?.map(|value| value.max(0) as u64);
    query.to_ms = optional_int(Some(arg), "toMs")?.map(|value| value.max(0) as u64);
    query.limit = optional_int(Some(arg), "limit")?.map(|value| value.max(0) as usize);
    Ok(query)
}

#[function]
fn version(ctx: CallContext) -> Result<JsValue> {
    let result = audit::run(Operation::Version, Value::Null, || Ok((core_version().to_string(), Summary::new(0, "version"))));
    match result {
        Ok(text) => ctx.env().string(&text),
        Err(error) => Err(core_error(error)),
    }
}

#[function]
fn capabilities(ctx: CallContext) -> Result<JsValue> {
    let result = audit::run(Operation::Capabilities, Value::Null, || {
        let report = core_capabilities();
        let summary = Summary::new(report.protocols.len() as u64, "host capabilities");
        Ok((report, summary))
    });
    match result {
        Ok(report) => value_to_js(&ctx, &report),
        Err(error) => Err(core_error(error)),
    }
}

#[function(js_name = "configureAudit")]
fn configure_audit(ctx: CallContext) -> Result<JsValue> {
    let arg = object_arg(&ctx)?;
    let actor = wae::audit::Actor {
        kind: actor_kind(arg.and_then(|obj| optional_string_prop(obj, "actorKind").ok().flatten()).as_deref()),
        id: arg.and_then(|obj| optional_string_prop(obj, "actorId").ok().flatten()).unwrap_or_else(|| String::from("unknown")),
        source: arg.and_then(|obj| optional_string_prop(obj, "actorSource").ok().flatten()).unwrap_or_else(|| String::from("local")),
    };
    let session = arg.and_then(|obj| optional_string_prop(obj, "session").ok().flatten());
    let host = arg.and_then(|obj| optional_string_prop(obj, "host").ok().flatten());
    audit::configure(actor, session, host);

    let object = ctx.env().object()?;
    object.set_named("ok", ctx.env().boolean(true)?)?;
    Ok(object)
}

#[function(js_name = "auditQuery")]
fn audit_query(ctx: CallContext) -> Result<JsValue> {
    let filter = query_from_arg(ctx.args().first().copied())?;
    let params = json!({ "capability": filter.capability, "limit": filter.limit });
    let result = audit::run(Operation::AuditQuery, params, || {
        let entries = audit::query(&filter)
            .map_err(|error| PlatformError::Failed { capability: String::from("audit-query"), message: error.to_string() })?;
        let summary = Summary::new(entries.len() as u64, "audit query");
        Ok((entries, summary))
    });
    match result {
        Ok(entries) => value_to_js(&ctx, &entries),
        Err(error) => Err(core_error(error)),
    }
}

#[function(js_name = "openGui")]
fn open_gui(ctx: CallContext) -> Result<JsValue> {
    let arg = object_arg(&ctx)?;
    let content_dir = resolve_gui_dir(arg.and_then(|obj| optional_string_prop(obj, "contentDir").ok().flatten()))?;
    let title = arg.and_then(|obj| optional_string_prop(obj, "title").ok().flatten()).unwrap_or_else(|| APP_TITLE.to_string());
    let params = json!({ "contentDir": &content_dir, "title": &title });

    let result = audit::run(Operation::GuiOpen, params, || {
        let backend = gui::open_window(&content_dir, &title)?;
        Ok((backend, Summary::new(1, "gui window opened")))
    });
    match result {
        Ok(backend) => {
            let object = ctx.env().object()?;
            object.set_named("backend", ctx.env().string(&backend)?)?;
            Ok(object)
        }
        Err(error) => Err(core_error(error)),
    }
}

#[function(js_name = "pumpGui")]
fn pump_gui(ctx: CallContext) -> Result<JsValue> {
    let result = audit::run(Operation::GuiPump, Value::Null, || Ok((gui::pump(), Summary::new(1, "gui event pump"))));
    match result {
        Ok(alive) => ctx.env().boolean(alive),
        Err(error) => Err(core_error(error)),
    }
}

#[function(js_name = "closeGui")]
fn close_gui(ctx: CallContext) -> Result<JsValue> {
    let result = audit::run(Operation::GuiClose, Value::Null, || {
        let closed = gui::close()?;
        Ok((closed, Summary::new(1, "gui window close")))
    });
    match result {
        Ok(closed) => ctx.env().boolean(closed),
        Err(error) => Err(core_error(error)),
    }
}

#[function(js_name = "guiStatus")]
fn gui_status(ctx: CallContext) -> Result<JsValue> {
    let result = audit::run(Operation::GuiStatus, Value::Null, || Ok((gui::status(), Summary::new(1, "gui status"))));
    match result {
        Ok(alive) => ctx.env().boolean(alive),
        Err(error) => Err(core_error(error)),
    }
}

register_module![version, capabilities, configure_audit, audit_query, open_gui, pump_gui, close_gui, gui_status];
