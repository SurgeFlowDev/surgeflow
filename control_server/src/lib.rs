use std::{marker::PhantomData, sync::Arc};

use adapter_types::{
    dependencies::control_server::ControlServerDependencies,
    senders::{EventSender, NewInstanceSender},
};
use aide::{OperationIo, axum::ApiRouter};
use axum::{Json, extract::State, http::StatusCode};
use axum_extra::routing::TypedPath;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use surgeflow_types::{
    __Step, __Workflow, InstanceEvent, Project, Workflow, WorkflowInstance, WorkflowInstanceId,
};

pub struct AppState<P: Project, E: EventSender<P>, I: NewInstanceSender<P>> {
    pub dependencies: ControlServerDependencies<P, E, I>,

    _marker: PhantomData<P>,
}

pub struct ArcAppState<P: Project, E: EventSender<P>, I: NewInstanceSender<P>>(
    pub Arc<AppState<P, E, I>>,
);

impl<P: Project, E: EventSender<P>, I: NewInstanceSender<P>> Clone for ArcAppState<P, E, I> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

pub async fn init_app_state<
    P: Project,
    EventSenderT: EventSender<P>,
    NewInstanceSenderT: NewInstanceSender<P>,
>(
    dependencies: ControlServerDependencies<P, EventSenderT, NewInstanceSenderT>,
) -> anyhow::Result<ArcAppState<P, EventSenderT, NewInstanceSenderT>> {
    Ok(ArcAppState(Arc::new(AppState {
        dependencies,

        _marker: PhantomData,
    })))
}
pub trait WorkflowControl<P: Project>: Workflow<P> {
    fn control_router<
        E: EventSender<P> + Send + Sync + 'static,
        N: NewInstanceSender<P> + Send + Sync + 'static,
    >() -> impl Future<Output = anyhow::Result<ApiRouter<ArcAppState<P, E, N>>>> + Send {
        async {
            let post_workflow_event_api_route = Self::post_workflow_event_api_route::<E, N>();
            let post_workflow_instance_api_route = Self::post_workflow_instance_api_route::<E, N>();

            let router = ApiRouter::new().nest(
                &format!("/workflow/{}", <Self as Workflow<P>>::NAME),
                ApiRouter::new()
                    .merge(post_workflow_instance_api_route)
                    .merge(post_workflow_event_api_route),
            );
            Ok(router)
        }
    }

    fn post_workflow_event_api_route<
        E: EventSender<P> + Send + Sync + 'static,
        N: NewInstanceSender<P> + Send + Sync + 'static,
    >() -> ApiRouter<ArcAppState<P, E, N>> {
        #[derive(TypedPath, Deserialize, JsonSchema, OperationIo)]
        #[typed_path("/{instance_id}/event")]
        pub struct PostWorkflowEvent {
            instance_id: WorkflowInstanceId,
        }

        #[derive(
            Debug,
            Serialize,
            Deserialize,
            JsonSchema,
            Clone,
            thiserror::Error,
            axum_thiserror::ErrorStatus,
            OperationIo,
        )]
        enum PostWorkflowEventError {
            #[error("could not queue event message")]
            #[status(axum::http::StatusCode::INTERNAL_SERVER_ERROR)]
            CouldntQueueEventMessage,
        }

        // more readable than a closure
        async fn handler<P: Project, T: Workflow<P>, E: EventSender<P>, N: NewInstanceSender<P>>(
            PostWorkflowEvent { instance_id }: PostWorkflowEvent,
            State(ArcAppState(state)): State<ArcAppState<P, E, N>>,
            Json(event): Json<<<T as Workflow<P>>::Step as __Step<P, T>>::Event>,
        ) -> Result<(), PostWorkflowEventError> {
            state
                .dependencies
                .event_sender
                .send(InstanceEvent {
                    event: event.into(),
                    instance_id,
                })
                .await
                .map_err(|_| PostWorkflowEventError::CouldntQueueEventMessage)?;
            Ok(())
        }

        ApiRouter::new().typed_post_with(handler::<P, Self, _, _>, |op| {
            op.description("Send event")
                .summary("Send event")
                .id("post-event")
                .tag(<Self as Workflow<P>>::NAME)
                .hidden(false)
        })
    }

    fn post_workflow_instance_api_route<
        E: EventSender<P> + Send + Sync + 'static,
        N: NewInstanceSender<P> + Send + Sync + 'static,
    >() -> ApiRouter<ArcAppState<P, E, N>> {
        #[derive(TypedPath, Deserialize, JsonSchema, OperationIo)]
        #[typed_path("/")]
        pub struct PostWorkflowInstance;

        #[derive(
            Debug,
            Serialize,
            Deserialize,
            JsonSchema,
            Clone,
            thiserror::Error,
            axum_thiserror::ErrorStatus,
            OperationIo,
        )]
        enum PostWorkflowInstanceError {
            #[error("could not create instance")]
            #[status(StatusCode::INTERNAL_SERVER_ERROR)]
            CouldntCreateInstance,
        }

        // more readable than a closure
        async fn handler<P: Project, T: Workflow<P>, E: EventSender<P>, N: NewInstanceSender<P>>(
            _: PostWorkflowInstance,
            State(ArcAppState(state)): State<ArcAppState<P, E, N>>,
        ) -> Result<Json<WorkflowInstanceId>, PostWorkflowInstanceError> {
            tracing::debug!("creating instance...");
            let external_id = WorkflowInstanceId::new();
            state
                .dependencies
                .new_instance_sender
                .send(WorkflowInstance {
                    workflow: <T as Workflow<P>>::WORKFLOW_STATIC.into(),
                    external_id,
                    // workflow_name: T::NAME.into(),
                })
                .await
                .map_err(|_| PostWorkflowInstanceError::CouldntCreateInstance)?;

            Ok(Json(external_id))
        }
        ApiRouter::new().typed_post_with(handler::<P, Self, _, _>, |op| {
            op.description("Create instance")
                .summary("Create instance")
                .id("post-workflow-instance")
                .tag(<Self as Workflow<P>>::NAME)
                .hidden(false)
        })
    }
}

impl<P: Project, T: Workflow<P>> WorkflowControl<P> for T {}

pub trait ProjectWorkflowControl<P: Project>: __Workflow<P> {
    fn control_router<NewEventSenderT: EventSender<P>, NewInstanceSenderT: NewInstanceSender<P>>()
    -> impl Future<
        Output = anyhow::Result<ApiRouter<ArcAppState<P, NewEventSenderT, NewInstanceSenderT>>>,
    > + Send;
}
