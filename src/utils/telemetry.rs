use once_cell::sync::Lazy;
use opentelemetry_prometheus::PrometheusExporter;
use prometheus::{Encoder, TextEncoder};

///static containing the prometeus expoter
pub static METER: Lazy<PrometheusExporter> = Lazy::new(init_meter);

///initialize the opentelemetry -> prometheus expoter
fn init_meter() -> PrometheusExporter {
    opentelemetry_prometheus::exporter().init()
}

///gather telemetry data and return them encoded for prometheus
pub async fn gather_telemetry() -> String {
    let encoder = TextEncoder::new();
    let metric_families = METER.registry().gather();
    let mut buffer = Vec::new();

    if let Err(e) = encoder.encode(&metric_families, &mut buffer) {
        error!("failed to encode telemetry: {e}");
    };
    match String::from_utf8(buffer) {
        Ok(resp) => {
            debug!("telemetry: {resp}");
            resp
        }
        Err(e) => {
            error!("failed to format telemetry to utf8: {e}");
            String::default()
        }
    }
}
