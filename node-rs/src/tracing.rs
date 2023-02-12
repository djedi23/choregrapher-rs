use tracing_subscriber::{prelude::*, EnvFilter};

pub fn init() {
  let registry = tracing_subscriber::registry().with(EnvFilter::from_default_env());
  #[cfg(not(feature = "forest"))]
  let registry = registry.with(tracing_subscriber::fmt::layer().compact());

  #[cfg(feature = "forest")]
  let registry = registry.with(tracing_forest::ForestLayer::default());

  #[cfg(feature = "otel")]
  let registry = registry.with({
    opentelemetry::global::set_text_map_propagator(opentelemetry_jaeger::Propagator::new());
    let tracer = opentelemetry_jaeger::new_agent_pipeline()
      .with_service_name("choregrapher")
      .install_simple()
      .unwrap();
    tracing_opentelemetry::layer()
      .with_tracer(tracer)
      .with_exception_fields(true)
  });

  #[cfg(feature = "tokio-console")]
  let registry = registry.with(console_subscriber::spawn());
  registry.init();
}
