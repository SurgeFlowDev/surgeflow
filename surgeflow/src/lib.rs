pub mod workers;
pub mod workflows;

#[cfg(not(any(
    feature = "active_step_worker",
    feature = "new_instance_worker",
    feature = "next_step_worker",
    feature = "new_event_worker",
    feature = "completed_step_worker",
    feature = "failed_step_worker",
    feature = "failed_instance_worker",
    feature = "completed_instance_worker",
    feature = "control_server"
)))]
compile_error!(
    "At least one worker feature must be enabled. Please enable one or more of the following features: active_step_worker, new_instance_worker, next_step_worker, new_event_worker, completed_step_worker, failed_step_worker, failed_instance_worker, completed_instance_worker, control_server."
);

pub use main_handler::main_handler;

#[cfg(any(
    feature = "active_step_worker",
    feature = "new_instance_worker",
    feature = "next_step_worker",
    feature = "new_event_worker",
    feature = "completed_step_worker",
    feature = "failed_step_worker",
    feature = "failed_instance_worker",
    feature = "completed_instance_worker",
    feature = "control_server"
))]
mod main_handler {
    use crate::workers::active_step_worker;
    use crate::workers::completed_instance_worker;
    use crate::workers::completed_step_worker;
    use crate::workers::control_server;
    use crate::workers::failed_instance_worker;
    use crate::workers::failed_step_worker;
    use crate::workers::new_event_worker;
    use crate::workers::new_instance_worker;
    use crate::workers::next_step_worker;
    use ::control_server::ProjectWorkflowControl;
    use adapter_types::dependencies::DependencyManager;
    use surgeflow_types::Project;
    use tokio::try_join;

    pub async fn main_handler<P: Project, D>(
        project: P,
        mut dependency_manager: D,
    ) -> anyhow::Result<()>
    where
        D: DependencyManager<P>,
        P::Workflow: ProjectWorkflowControl,
    {
        try_join!(
            #[cfg(feature = "control_server")]
            control_server::main::<P, _, _>(
                dependency_manager
                    .control_server_dependencies()
                    .await
                    .expect("Failed to get control server dependencies")
            ),
            #[cfg(feature = "active_step_worker")]
            active_step_worker::main::<P, _, _, _, _, _>(
                dependency_manager
                    .active_step_worker_dependencies()
                    .await
                    .expect("Failed to get active step worker dependencies"),
                project,
            ),
            #[cfg(feature = "new_instance_worker")]
            new_instance_worker::main::<P, _, _, _>(
                dependency_manager
                    .new_instance_worker_dependencies()
                    .await
                    .expect("Failed to get new instance worker dependencies")
            ),
            #[cfg(feature = "next_step_worker")]
            next_step_worker::main::<P, _, _, _, _>(
                dependency_manager
                    .next_step_worker_dependencies()
                    .await
                    .expect("Failed to get next step worker dependencies")
            ),
            #[cfg(feature = "new_event_worker")]
            new_event_worker::main::<P, _, _, _>(
                dependency_manager
                    .new_event_worker_dependencies()
                    .await
                    .expect("Failed to get new event worker dependencies")
            ),
            #[cfg(feature = "completed_step_worker")]
            completed_step_worker::main::<P, _, _, _>(
                dependency_manager
                    .completed_step_worker_dependencies()
                    .await
                    .expect("Failed to get completed step worker dependencies")
            ),
            #[cfg(feature = "failed_step_worker")]
            failed_step_worker::main::<P, _, _, _>(
                dependency_manager
                    .failed_step_worker_dependencies()
                    .await
                    .expect("Failed to get failed step worker dependencies")
            ),
            #[cfg(feature = "failed_instance_worker")]
            failed_instance_worker::main::<P, _>(
                dependency_manager
                    .failed_instance_worker_dependencies()
                    .await
                    .expect("Failed to get failed instance worker dependencies")
            ),
            #[cfg(feature = "completed_instance_worker")]
            completed_instance_worker::main::<P, _>(
                dependency_manager
                    .completed_instance_worker_dependencies()
                    .await
                    .expect("Failed to get completed instance worker dependencies")
            ),
        )?;

        Ok(())
    }
}
