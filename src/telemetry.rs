use tracing::{subscriber::set_global_default, Subscriber};
use tracing_bunyan_formatter::BunyanFormattingLayer;
use tracing_subscriber::{fmt::MakeWriter, layer::SubscriberExt, EnvFilter, Registry};

pub fn get_subscriber<Sink>(
    name: String,
    env_filter: String,
    sink: Sink,
) -> impl Subscriber + Send + Sync
where
    //NOTE: This syntax is
    Sink: for<'a> MakeWriter<'a> + Send + Sync + 'static,
{
    let env_filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(env_filter));
    //NOTE: This is layer that is going to output the resulting tracing event records
    let formatting_layer = BunyanFormattingLayer::new(name, sink);
    Registry::default()
        .with(env_filter)
        .with(tracing_bunyan_formatter::JsonStorageLayer) //I assume this will create a `default instance`
        .with(formatting_layer)
}

pub fn init_subscriber(subscriber: impl Subscriber + Send + Sync) {
    //WARN: Register a subscriber as global default to process span data.
    //It should only be called once!
    set_global_default(subscriber).expect("Failed to set subscriber");
}
