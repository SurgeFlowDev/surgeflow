use crate::{
    workers::{
        adapters::dependencies::{ControlServerDependencyProvider, control_server::{ControlServerContext, ControlServerDependencies}},
        azure_adapter::{
            dependencies::{AzureDependencyManager, AzureAdapterConfig},
            managers::AzureServiceBusWorkflowInstanceManager, 
            senders::AzureServiceBusEventSender,
        },
    },
    workflows::Project,
};

pub struct AzureServiceBusControlServerContext<P: Project> {
    dependency_manager: AzureDependencyManager,
    _marker: std::marker::PhantomData<P>,
}

impl<P: Project> AzureServiceBusControlServerContext<P> {
    pub fn new(config: AzureAdapterConfig) -> Self {
        Self {
            dependency_manager: AzureDependencyManager::new(config),
            _marker: std::marker::PhantomData,
        }
    }
}

impl<P: Project> ControlServerContext<P> for AzureServiceBusControlServerContext<P> {
    type EventSender = AzureServiceBusEventSender<P>;
    type InstanceManager = AzureServiceBusWorkflowInstanceManager<P>;
    
    async fn dependencies() -> anyhow::Result<ControlServerDependencies<P, Self::EventSender, Self::InstanceManager>> {
        // Create a default config - this could be improved by making it configurable
        let config = AzureAdapterConfig {
            service_bus_connection_string: std::env::var("AZURE_SERVICE_BUS_CONNECTION_STRING")
                .expect("AZURE_SERVICE_BUS_CONNECTION_STRING must be set"),
            cosmos_connection_string: std::env::var("AZURE_COSMOS_CONNECTION_STRING")
                .expect("AZURE_COSMOS_CONNECTION_STRING must be set"),
            new_instance_queue_suffix: "new-instances".to_string(),
            next_step_queue_suffix: "next-steps".to_string(),
            completed_instance_queue_suffix: "completed-instances".to_string(),
            completed_step_queue_suffix: "completed-steps".to_string(),
            active_step_queue_suffix: "active-steps".to_string(),
            failed_instance_queue_suffix: "failed-instances".to_string(),
            failed_step_queue_suffix: "failed-steps".to_string(),
            new_event_queue_suffix: "new-events".to_string(),
            pg_connection_string: std::env::var("DATABASE_URL")
                .expect("DATABASE_URL must be set"),
        };
        
        let mut dependency_manager = AzureDependencyManager::new(config);
        dependency_manager.control_server_dependencies().await
    }
}
