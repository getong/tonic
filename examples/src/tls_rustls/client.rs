//! This examples shows how you can combine `hyper-rustls` and `tonic` to
//! provide a custom `ClientConfig` for the tls configuration.

pub mod pb {
    tonic::include_proto!("/grpc.examples.unaryecho");
}

use hyper::Uri;
use hyper_util::{client::legacy::connect::HttpConnector, rt::TokioExecutor};
use pb::{echo_client::EchoClient, EchoRequest};
use tokio_rustls::rustls::{
    pki_types::{pem::PemObject as _, CertificateDer},
    {ClientConfig, RootCertStore},
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let data_dir = std::path::PathBuf::from_iter([std::env!("CARGO_MANIFEST_DIR"), "data"]);
    let fd = std::fs::File::open(data_dir.join("tls/ca.pem"))?;

    let mut roots = RootCertStore::empty();

    let mut buf = std::io::BufReader::new(&fd);
    let certs = CertificateDer::pem_reader_iter(&mut buf).collect::<Result<Vec<_>, _>>()?;
    roots.add_parsable_certificates(certs.into_iter());

    let tls = ClientConfig::builder()
        .with_root_certificates(roots)
        .with_no_client_auth();

    let mut http = HttpConnector::new();
    http.enforce_http(false);

    // We have to do some wrapping here to map the request type from
    // `https://example.com` -> `https://[::1]:50051` because `rustls`
    // doesn't accept ip's as `ServerName`.
    let connector = tower::ServiceBuilder::new()
        .layer_fn(move |s| {
            let tls = tls.clone();

            hyper_rustls::HttpsConnectorBuilder::new()
                .with_tls_config(tls)
                .https_or_http()
                .enable_http2()
                .wrap_connector(s)
        })
        // Since our cert is signed with `example.com` but we actually want to connect
        // to a local server we will override the Uri passed from the `HttpsConnector`
        // and map it to the correct `Uri` that will connect us directly to the local server.
        .map_request(|_| Uri::from_static("https://[::1]:50051"))
        .service(http);

    let client = hyper_util::client::legacy::Client::builder(TokioExecutor::new()).build(connector);

    // Using `with_origin` will let the codegenerated client set the `scheme` and
    // `authority` from the provided `Uri`.
    let uri = Uri::from_static("https://example.com");
    let mut client = EchoClient::with_origin(client, uri);

    let request = tonic::Request::new(EchoRequest {
        message: "hello".into(),
    });

    let response = client.unary_echo(request).await?;

    println!("RESPONSE={response:?}");

    Ok(())
}
