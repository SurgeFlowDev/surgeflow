struct Workflow1 {}

#[workflow("workflow_1")]
impl Workflow for Workflow1 {
    workflow_event! {
        Event0,
        Event1,
    }
    workflow_step! {
        Step0,
        Step1,
    }
    fn entrypoint() -> StepWithSettings<Self::Step> {
        StepWithSettings {
            step: Step0 {}.into(),
            settings: StepSettings { max_retries: 0 },
        }
    }
}
