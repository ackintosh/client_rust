use prometheus_client::encoding::proto::EncodeLabel;
use prometheus_client::encoding::proto::{encode, EncodeProtobuf};
use prometheus_client::metrics::counter::Counter;
use prometheus_client::metrics::family::Family;
use prometheus_client::registry::Registry;
use std::fmt::{Display, Formatter};

#[test]
fn structs() {
    #[derive(Clone, Hash, PartialEq, Eq, EncodeProtobuf)]
    struct Labels {
        method: Method,
        path: String,
    }

    #[derive(Clone, Hash, PartialEq, Eq)]
    enum Method {
        Get,
        #[allow(dead_code)]
        Put,
    }

    impl Display for Method {
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            match self {
                Method::Get => f.write_str("Get"),
                Method::Put => f.write_str("Put"),
            }
        }
    }

    let mut registry = Registry::default();
    let family = Family::<Labels, Counter>::default();
    registry.register("my_counter", "This is my counter", family.clone());

    // Record a single HTTP GET request.
    family
        .get_or_create(&Labels {
            method: Method::Get,
            path: "/metrics".to_string(),
        })
        .inc();

    // Encode all metrics in the registry in the OpenMetrics protobuf format.
    let mut metric_set = encode(&registry);
    let mut family: prometheus_client::encoding::proto::MetricFamily =
        metric_set.metric_families.pop().unwrap();
    let metric: prometheus_client::encoding::proto::Metric = family.metrics.pop().unwrap();

    let method = &metric.labels[0];
    assert_eq!("method", method.name);
    assert_eq!("Get", method.value);

    let path = &metric.labels[1];
    assert_eq!("path", path.name);
    assert_eq!("/metrics", path.value);
}

#[test]
fn enums() {
    #[derive(Clone, Hash, PartialEq, Eq, EncodeProtobuf)]
    enum Method {
        Get,
        #[allow(dead_code)]
        Put,
    }

    let mut registry = Registry::default();
    let family = Family::<Method, Counter>::default();
    registry.register("my_counter", "This is my counter", family.clone());

    // Record a single HTTP GET request.
    family.get_or_create(&Method::Get).inc();

    // Encode all metrics in the registry in the OpenMetrics protobuf format.
    let mut metric_set = encode(&registry);
    let mut family: prometheus_client::encoding::proto::MetricFamily =
        metric_set.metric_families.pop().unwrap();
    let metric: prometheus_client::encoding::proto::Metric = family.metrics.pop().unwrap();

    let label = &metric.labels[0];
    assert_eq!("Method", label.name);
    assert_eq!("Get", label.value);
}

#[test]
fn remap_keyword_identifiers() {
    #[derive(EncodeProtobuf, Hash, Clone, Eq, PartialEq)]
    struct Labels {
        // `r#type` is problematic as `r#` is not a valid OpenMetrics label name
        // but one needs to use keyword identifier syntax (aka. raw identifiers)
        // as `type` is a keyword.
        //
        // Test makes sure `r#type` is replaced by `type` in the OpenMetrics
        // output.
        r#type: u64,
    }

    let labels = Labels { r#type: 42 }.encode();

    assert_eq!("type", labels[0].name);
    assert_eq!("42", labels[0].value);
}
