use envoy_proxy_dynamic_modules_rust_sdk::*;

mod body_based_routing;

declare_init_functions!(init, new_http_filter_config_fn);

/// This implements the [`envoy_proxy_dynamic_modules_rust_sdk::ProgramInitFunction`].
///
/// This is called exactly once when the module is loaded. It can be used to
/// initialize global state as well as check the runtime environment to ensure that
/// the module is running in a supported environment.
///
/// Returning `false` will cause Envoy to reject the config hence the
/// filter will not be loaded.
fn init() -> bool {
    eprintln!("[BODY_TO_HEADER] Module initialized");
    true
}

/// This implements the [`envoy_proxy_dynamic_modules_rust_sdk::NewHttpFilterConfigFunction`].
///
/// This is the entrypoint every time a new HTTP filter is created via the DynamicModuleFilter config.
///
/// Each argument matches the corresponding argument in the Envoy config here:
/// https://www.envoyproxy.io/docs/envoy/latest/api-v3/extensions/dynamic_modules/v3/dynamic_modules.proto#envoy-v3-api-msg-extensions-dynamic-modules-v3-dynamicmoduleconfig
///
/// Returns None if the filter name or config is determined to be invalid by each filter's `new` function.
fn new_http_filter_config_fn<EC: EnvoyHttpFilterConfig, EHF: EnvoyHttpFilter>(
    _envoy_filter_config: &mut EC,
    filter_name: &str,
    filter_config: &[u8],
) -> Option<Box<dyn HttpFilterConfig<EC, EHF>>> {
    let filter_config = std::str::from_utf8(filter_config).unwrap_or("");
    eprintln!("[BODY_TO_HEADER] Creating filter: {}, config: {}", filter_name, filter_config);
    
    match filter_name {
        "body_based_routing" => Some(Box::new(body_based_routing::FilterConfig::new(filter_config))),
        _ => {
            eprintln!("[BODY_TO_HEADER] Unknown filter name: {}", filter_name);
            None
        }
    }
} 