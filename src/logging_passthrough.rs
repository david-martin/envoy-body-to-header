use envoy_proxy_dynamic_modules_rust_sdk::*;
use serde::{Deserialize, Serialize};

/// This implements the [`envoy_proxy_dynamic_modules_rust_sdk::HttpFilterConfig`] trait.
///
/// The trait corresponds to a Envoy filter chain configuration.
#[derive(Serialize, Deserialize, Debug)]
pub struct FilterConfig {
    #[serde(default)]
    debug: bool,
}

impl FilterConfig {
    /// This is the constructor for the [`FilterConfig`].
    ///
    /// filter_config is the filter config from the Envoy config here:
    /// https://www.envoyproxy.io/docs/envoy/latest/api-v3/extensions/dynamic_modules/v3/dynamic_modules.proto#envoy-v3-api-msg-extensions-dynamic-modules-v3-dynamicmoduleconfig
    pub fn new(filter_config: &str) -> Self {
        eprintln!("[BODY_TO_HEADER] FilterConfig created with config: {}", filter_config);
        
        let config = if filter_config.trim().is_empty() {
            // Default config if empty
            FilterConfig { debug: false }
        } else {
            match serde_json::from_str::<FilterConfig>(filter_config) {
                Ok(cfg) => {
                    eprintln!("[BODY_TO_HEADER] Parsed config successfully: debug={}", cfg.debug);
                    cfg
                }
                Err(err) => {
                    eprintln!("[BODY_TO_HEADER] Error parsing filter config, using defaults: {}", err);
                    FilterConfig { debug: false }
                }
            }
        };
        
        config
    }
}

impl<EC: EnvoyHttpFilterConfig, EHF: EnvoyHttpFilter> HttpFilterConfig<EC, EHF> for FilterConfig {
    /// This is called for each new HTTP filter.
    fn new_http_filter(&mut self, _envoy: &mut EC) -> Box<dyn HttpFilter<EHF>> {
        eprintln!("[BODY_TO_HEADER] Creating new HTTP filter instance");
        Box::new(Filter::new())
    }
}

/// This implements the [`envoy_proxy_dynamic_modules_rust_sdk::HttpFilter`] trait.
///
/// This is a passthrough filter that logs at each stage of request processing.
pub struct Filter {
    request_id: String,
}

impl Filter {
    pub fn new() -> Self {
        let request_id = format!("req_{}", std::process::id());
        eprintln!("[BODY_TO_HEADER] [{}] Filter created", request_id);
        Self { request_id }
    }

    fn log_headers(&self, stage: &str, headers: &[(EnvoyBuffer, EnvoyBuffer)]) {
        eprintln!("[BODY_TO_HEADER] [{}] === {} HEADERS ===", self.request_id, stage);
        for (key, value) in headers {
            if let (Ok(key_str), Ok(value_str)) = (
                std::str::from_utf8(key.as_slice()),
                std::str::from_utf8(value.as_slice())
            ) {
                eprintln!("[BODY_TO_HEADER] [{}]   {}: {}", self.request_id, key_str, value_str);
            }
        }
        eprintln!("[BODY_TO_HEADER] [{}] ========================", self.request_id);
    }

    fn log_body(&self, stage: &str, body: Option<&[u8]>, end_of_stream: bool) {
        eprintln!("[BODY_TO_HEADER] [{}] === {} BODY ===", self.request_id, stage);
        eprintln!("[BODY_TO_HEADER] [{}] End of stream: {}", self.request_id, end_of_stream);
        if let Some(body_data) = body {
            eprintln!("[BODY_TO_HEADER] [{}] Body length: {} bytes", self.request_id, body_data.len());
            if body_data.len() > 0 {
                match std::str::from_utf8(body_data) {
                    Ok(body_str) => {
                        let preview = if body_str.len() > 200 {
                            format!("{}...", &body_str[..200])
                        } else {
                            body_str.to_string()
                        };
                        eprintln!("[BODY_TO_HEADER] [{}] Body preview: {}", self.request_id, preview);
                    }
                    Err(_) => {
                        eprintln!("[BODY_TO_HEADER] [{}] Body contains non-UTF8 data", self.request_id);
                    }
                }
            }
        } else {
            eprintln!("[BODY_TO_HEADER] [{}] No body data", self.request_id);
        }
        eprintln!("[BODY_TO_HEADER] [{}] =================", self.request_id);
    }

    fn log_trailers(&self, stage: &str, trailers: &[(EnvoyBuffer, EnvoyBuffer)]) {
        eprintln!("[BODY_TO_HEADER] [{}] === {} TRAILERS ===", self.request_id, stage);
        for (key, value) in trailers {
            if let (Ok(key_str), Ok(value_str)) = (
                std::str::from_utf8(key.as_slice()),
                std::str::from_utf8(value.as_slice())
            ) {
                eprintln!("[BODY_TO_HEADER] [{}]   {}: {}", self.request_id, key_str, value_str);
            }
        }
        eprintln!("[BODY_TO_HEADER] [{}] ==========================", self.request_id);
    }
}

/// This implements the [`envoy_proxy_dynamic_modules_rust_sdk::HttpFilter`] trait.
impl<EHF: EnvoyHttpFilter> HttpFilter<EHF> for Filter {
    fn on_request_headers(
        &mut self,
        envoy_filter: &mut EHF,
        end_of_stream: bool,
    ) -> abi::envoy_dynamic_module_type_on_http_filter_request_headers_status {
        eprintln!("[BODY_TO_HEADER] [{}] on_request_headers called (end_of_stream: {})", self.request_id, end_of_stream);
        
        let headers = envoy_filter.get_request_headers();
        self.log_headers("REQUEST", &headers);
        
        abi::envoy_dynamic_module_type_on_http_filter_request_headers_status::Continue
    }

    fn on_request_body(
        &mut self,
        envoy_filter: &mut EHF,
        end_of_stream: bool,
    ) -> abi::envoy_dynamic_module_type_on_http_filter_request_body_status {
        eprintln!("[BODY_TO_HEADER] [{}] on_request_body called (end_of_stream: {})", self.request_id, end_of_stream);
        
        if let Some(body_buffers) = envoy_filter.get_request_body() {
            // Collect body data from all buffers
            let mut body_data = Vec::new();
            for buffer in body_buffers {
                body_data.extend_from_slice(buffer.as_slice());
            }
            self.log_body("REQUEST", Some(&body_data), end_of_stream);
        } else {
            self.log_body("REQUEST", None, end_of_stream);
        }
        
        abi::envoy_dynamic_module_type_on_http_filter_request_body_status::Continue
    }

    fn on_request_trailers(
        &mut self,
        envoy_filter: &mut EHF,
    ) -> abi::envoy_dynamic_module_type_on_http_filter_request_trailers_status {
        eprintln!("[BODY_TO_HEADER] [{}] on_request_trailers called", self.request_id);
        
        let trailers = envoy_filter.get_request_trailers();
        self.log_trailers("REQUEST", &trailers);
        
        abi::envoy_dynamic_module_type_on_http_filter_request_trailers_status::Continue
    }

    fn on_response_headers(
        &mut self,
        envoy_filter: &mut EHF,
        end_of_stream: bool,
    ) -> abi::envoy_dynamic_module_type_on_http_filter_response_headers_status {
        eprintln!("[BODY_TO_HEADER] [{}] on_response_headers called (end_of_stream: {})", self.request_id, end_of_stream);
        
        let headers = envoy_filter.get_response_headers();
        self.log_headers("RESPONSE", &headers);
        
        abi::envoy_dynamic_module_type_on_http_filter_response_headers_status::Continue
    }

    fn on_response_body(
        &mut self,
        envoy_filter: &mut EHF,
        end_of_stream: bool,
    ) -> abi::envoy_dynamic_module_type_on_http_filter_response_body_status {
        eprintln!("[BODY_TO_HEADER] [{}] on_response_body called (end_of_stream: {})", self.request_id, end_of_stream);
        
        if let Some(body_buffers) = envoy_filter.get_response_body() {
            // Collect body data from all buffers
            let mut body_data = Vec::new();
            for buffer in body_buffers {
                body_data.extend_from_slice(buffer.as_slice());
            }
            self.log_body("RESPONSE", Some(&body_data), end_of_stream);
        } else {
            self.log_body("RESPONSE", None, end_of_stream);
        }
        
        abi::envoy_dynamic_module_type_on_http_filter_response_body_status::Continue
    }

    fn on_response_trailers(
        &mut self,
        envoy_filter: &mut EHF,
    ) -> abi::envoy_dynamic_module_type_on_http_filter_response_trailers_status {
        eprintln!("[BODY_TO_HEADER] [{}] on_response_trailers called", self.request_id);
        
        let trailers = envoy_filter.get_response_trailers();
        self.log_trailers("RESPONSE", &trailers);
        
        abi::envoy_dynamic_module_type_on_http_filter_response_trailers_status::Continue
    }
}

impl Drop for Filter {
    fn drop(&mut self) {
        eprintln!("[BODY_TO_HEADER] [{}] Filter dropped", self.request_id);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_filter_config() {
        // Test with valid JSON config
        let config = FilterConfig::new(r#"{"debug": true}"#);
        assert_eq!(config.debug, true);
        
        // Test with empty config (uses defaults)
        let config = FilterConfig::new("");
        assert_eq!(config.debug, false);
        
        // Test with invalid JSON (uses defaults)
        let config = FilterConfig::new("invalid json");
        assert_eq!(config.debug, false);
    }

    #[test]
    fn test_filter_creation() {
        let filter = Filter::new();
        assert!(!filter.request_id.is_empty());
    }
} 