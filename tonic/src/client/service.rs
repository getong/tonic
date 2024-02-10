use http_body::Body;
use std::future::Future;
use std::task::{Context, Poll};
use tower_service::Service;

/// Definition of the gRPC trait alias for [`tower_service`].
///
/// This trait enforces that all tower services provided to [`Grpc`] implements
/// the correct traits.
///
/// [`Grpc`]: ../client/struct.Grpc.html
/// [`tower_service`]: https://docs.rs/tower-service
pub trait GrpcService<ReqIncoming> {
    /// Responses body given by the service.
    type ResponseIncoming: Incoming;
    /// Errors produced by the service.
    type Error: Into<crate::Error>;
    /// The future response value.
    type Future: Future<Output = Result<http::Response<Self::ResponseIncoming>, Self::Error>>;

    /// Returns `Ready` when the service is able to process requests.
    ///
    /// Reference [`Service::poll_ready`].
    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>>;

    /// Process the request and return the response asynchronously.
    ///
    /// Reference [`Service::call`].
    fn call(&mut self, request: http::Request<ReqIncoming>) -> Self::Future;
}

impl<T, ReqIncoming, ResIncoming> GrpcService<ReqIncoming> for T
where
    T: Service<http::Request<ReqIncoming>, Response = http::Response<ResIncoming>>,
    T::Error: Into<crate::Error>,
    ResIncoming: Incoming,
    <ResIncoming as Incoming>::Error: Into<crate::Error>,
{
    type ResponseIncoming = ResIncoming;
    type Error = T::Error;
    type Future = T::Future;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Service::poll_ready(self, cx)
    }

    fn call(&mut self, request: http::Request<ReqIncoming>) -> Self::Future {
        Service::call(self, request)
    }
}
