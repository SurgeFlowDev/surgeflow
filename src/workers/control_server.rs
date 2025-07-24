use aide::{
    axum::{ApiRouter, IntoApiResponse, routing::get_with},
    openapi::{Info, OpenApi},
    scalar::Scalar,
};
use axum::{Extension, Json, ServiceExt, extract::Request};
use tokio::net::TcpListener;
use tower::Layer;
use tower_http::normalize_path::NormalizePathLayer;

use crate::{
    workers::adapters::{
        dependencies::control_server::ControlServerDependencies,
        senders::{EventSender, NewInstanceSender},
    },
    workflows::{Project, ProjectWorkflow, init_app_state},
};

pub async fn main<P, EventSenderT, NewInstanceSenderT>(
    dependencies: ControlServerDependencies<P, EventSenderT, NewInstanceSenderT>,
) -> anyhow::Result<()>
where
    P: Project,
    EventSenderT: EventSender<P> + std::marker::Send + std::marker::Sync + 'static,
    NewInstanceSenderT: NewInstanceSender<P> + std::marker::Send + std::marker::Sync + 'static,
{
    let app_state = init_app_state(dependencies).await?;
    let router = P::Workflow::control_router()
        .await?
        .with_state(app_state.clone());

    serve(router).await
}

async fn serve(router: ApiRouter) -> anyhow::Result<()> {
    let router = ApiRouter::new().merge(router);

    let mut api = base_open_api();

    let router = if cfg!(debug_assertions) {
        let router = router
            .api_route(
                "/openapi.json",
                get_with(serve_api, |op| op.summary("OpenAPI Spec").hidden(false)),
            )
            .route("/docs", Scalar::new("/openapi.json").axum_route());
        router.finish_api(&mut api).layer(Extension(api))
    } else {
        router.into()
    };
    let router = NormalizePathLayer::trim_trailing_slash().layer(router);
    let router = ServiceExt::<Request>::into_make_service(router);

    let listener = TcpListener::bind(&format!("0.0.0.0:{}", 8080)).await?;
    axum::serve(listener, router).await?;
    Ok(())
}

pub fn base_open_api() -> OpenApi {
    OpenApi {
        info: Info {
            description: Some("API".to_string()),
            ..Info::default()
        },
        ..OpenApi::default()
    }
}

async fn serve_api(Extension(api): Extension<OpenApi>) -> impl IntoApiResponse {
    Json(api)
}
