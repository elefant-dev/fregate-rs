use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use fregate::observability::EventFormatter;
use std::io;
use time::format_description::well_known::Rfc3339;
use tracing::subscriber::with_default;
use tracing_subscriber::fmt::format::{DefaultFields, Format, Json, JsonFields};
use tracing_subscriber::fmt::time::UtcTime;
use tracing_subscriber::fmt::{MakeWriter, Subscriber};
use tracing_subscriber::EnvFilter;

fn benchmark(c: &mut Criterion) {
    let i = 1000;
    let data = unsafe { String::from_utf8_unchecked(vec![b'X'; 3000]) };

    let mut group = c.benchmark_group("Event Formatter");

    group.bench_with_input(
        BenchmarkId::new("Default tracing", i),
        &(i, &data),
        |b, (i, data)| {
            let subscriber = tracing_subscriber();

            with_default(subscriber, || {
                b.iter(|| {
                    for _ in 0..*i {
                        tracing::info!(secret = "12345", "message = {data}");
                    }
                });
            });
        },
    );
    group.bench_with_input(
        BenchmarkId::new("Fregate None", i),
        &(i, &data),
        |b, (i, data)| {
            let subscriber = subscriber(EventFormatter::new_with_limits(None));

            with_default(subscriber, || {
                b.iter(|| {
                    for _ in 0..*i {
                        tracing::info!(secret = "12345", "message = {data}");
                    }
                });
            });
        },
    );
    group.bench_with_input(
        BenchmarkId::new("Fregate Some(256)", i),
        &(i, &data),
        |b, (i, data)| {
            let subscriber = subscriber(EventFormatter::new_with_limits(Some(256)));

            with_default(subscriber, || {
                b.iter(|| {
                    for _ in 0..*i {
                        tracing::info!(secret = "12345", "message = {data}");
                    }
                });
            });
        },
    );
    group.finish();
}

criterion_group!(benches, benchmark);
criterion_main!(benches);

#[derive(Clone, Debug)]
struct MockWriter;

#[derive(Clone, Debug)]
struct MakeMockWriter;

impl MakeMockWriter {
    fn new() -> Self {
        Self {}
    }
}

impl MockWriter {
    fn new() -> Self {
        Self {}
    }
}

impl io::Write for MockWriter {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl<'a> MakeWriter<'a> for MakeMockWriter {
    type Writer = MockWriter;

    fn make_writer(&'a self) -> Self::Writer {
        MockWriter::new()
    }
}

fn subscriber(
    formatter: EventFormatter,
) -> Subscriber<DefaultFields, EventFormatter, EnvFilter, MakeMockWriter> {
    Subscriber::builder()
        .event_format(formatter)
        .with_writer(MakeMockWriter::new())
        .with_env_filter("info")
        .finish()
}

fn tracing_subscriber(
) -> Subscriber<JsonFields, Format<Json, UtcTime<Rfc3339>>, EnvFilter, MakeMockWriter> {
    tracing_subscriber::fmt()
        .json()
        .with_timer::<_>(UtcTime::rfc_3339())
        .with_writer(MakeMockWriter::new())
        .flatten_event(true)
        .with_target(true)
        .with_current_span(false)
        .with_env_filter("info")
        .finish()
}
