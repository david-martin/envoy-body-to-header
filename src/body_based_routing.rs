use envoy_proxy_dynamic_modules_rust_sdk::*;
use serde::{Deserialize, Serialize};

/// Configuration for the body-based routing filter.
#[derive(Serialize, Deserialize, Debug)]
pub struct FilterConfig {
    #[serde(default)]
    debug: bool,
}

impl FilterConfig {
    /// Creates a new FilterConfig from JSON configuration.
    pub fn new(filter_config: &str) -> Self {
        if filter_config.trim().is_empty() {
            FilterConfig { debug: false }
        } else {
            serde_json::from_str::<FilterConfig>(filter_config)
                .unwrap_or_else(|_| FilterConfig { debug: false })
        }
    }
}

impl<EC: EnvoyHttpFilterConfig, EHF: EnvoyHttpFilter> HttpFilterConfig<EC, EHF> for FilterConfig {
    fn new_http_filter(&mut self, _envoy: &mut EC) -> Box<dyn HttpFilter<EHF>> {
        Box::new(Filter::new())
    }
}

/// Body-based routing filter that analyzes request bodies and sets routing headers.
/// 
/// MEMORY CONSIDERATIONS:
/// - Buffers complete request bodies in memory during analysis
/// - Memory usage scales with request body size
/// - Consider implementing body size limits for production use
/// 
/// LATENCY CONSIDERATIONS:
/// - Pauses request processing until complete body is available
/// - JSON parsing adds computational overhead
/// - Route cache clearing forces re-evaluation (small cost)
pub struct Filter;

impl Filter {
    pub fn new() -> Self {
        Self
    }
}

impl<EHF: EnvoyHttpFilter> HttpFilter<EHF> for Filter {
    fn on_request_headers(
        &mut self,
        _envoy_filter: &mut EHF,
        end_of_stream: bool,
    ) -> abi::envoy_dynamic_module_type_on_http_filter_request_headers_status {
        
        // CRITICAL: For requests with bodies, we must pause header processing here.
        // If we don't pause, Envoy will make routing decisions before we can analyze
        // the body content and set our routing header. StopIteration prevents
        // upstream connection establishment until body analysis is complete.
        if !end_of_stream {
            return abi::envoy_dynamic_module_type_on_http_filter_request_headers_status::StopIteration;
        }
        
        // No body expected - continue with default routing
        abi::envoy_dynamic_module_type_on_http_filter_request_headers_status::Continue
    }

    fn on_request_body(
        &mut self,
        envoy_filter: &mut EHF,
        end_of_stream: bool,
    ) -> abi::envoy_dynamic_module_type_on_http_filter_request_body_status {
        
        // MEMORY OPTIMIZATION: Buffer body chunks until we have the complete body.
        // StopIterationAndBuffer tells Envoy to accumulate all body data before
        // calling us again with end_of_stream=true. This avoids complex state
        // management but uses more memory for large bodies.
        if !end_of_stream {
            return abi::envoy_dynamic_module_type_on_http_filter_request_body_status::StopIterationAndBuffer;
        }
        
        // Default route - most requests go here for optimal performance
        let mut route_to = "echo1";
        
        // PERFORMANCE CRITICAL: Only process body if we have data to avoid unnecessary work
        if let Some(body_buffers) = envoy_filter.get_request_body() {
            let mut body_data = Vec::new();
            for buffer in body_buffers {
                body_data.extend_from_slice(buffer.as_slice());
            }
            
            // LATENCY CONSIDERATION: JSON parsing adds overhead but enables intelligent routing
            if !body_data.is_empty() {
                if let Ok(body_str) = std::str::from_utf8(&body_data) {
                    if body_str.contains("\"method\"") {
                        if let Ok(json_value) = serde_json::from_str::<serde_json::Value>(body_str) {
                            if let Some(method) = json_value.get("method").and_then(|m| m.as_str()) {
                                if method.contains("echo2") {
                                    route_to = "echo2";
                                }
                            }
                        }
                    }
                }
            }
        }
        
        // ROUTING CRITICAL: Set the header that our Envoy route configuration will match against
        envoy_filter.set_request_header("x-route-to", route_to.as_bytes());
        
        // ESSENTIAL: clear_route_cache() forces Envoy to re-evaluate routing decisions
        // after we've set our routing header. Without this call, Envoy may use
        // cached routing decisions made before our header was available, causing
        // requests to be routed incorrectly.
        envoy_filter.clear_route_cache();
        
        // Resume normal request processing with our routing header in place
        abi::envoy_dynamic_module_type_on_http_filter_request_body_status::Continue
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_filter_config() {
        let config = FilterConfig::new(r#"{"debug": true}"#);
        assert_eq!(config.debug, true);
        
        let config = FilterConfig::new("");
        assert_eq!(config.debug, false);
        
        let config = FilterConfig::new("invalid json");
        assert_eq!(config.debug, false);
    }

    #[test]
    fn test_filter_creation() {
        let _filter = Filter::new();
        // Filter creation should succeed
    }
} 