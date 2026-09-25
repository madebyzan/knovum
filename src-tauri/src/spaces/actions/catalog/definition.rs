use crate::ai::contracts::{ExecutionCapability, RoutingPurpose};

use super::super::{
    ParameterSchemaFactory, SpaceActionApprovalPolicy, SpaceActionDefinition,
    SpaceActionExecutionMode, SpaceActionImpact, SpaceActionInputConsistency, SpaceActionInputRole,
    SpaceActionOutputContract,
};

struct ActionBinding {
    capability: Option<ExecutionCapability>,
    purpose: Option<RoutingPurpose>,
}

pub(super) const fn action(
    id: &'static str,
    execution_capability: Option<ExecutionCapability>,
    routing_purpose: Option<RoutingPurpose>,
    input_roles: &'static [SpaceActionInputRole],
    parameters_schema: ParameterSchemaFactory,
    output_contract: SpaceActionOutputContract,
) -> SpaceActionDefinition {
    definition(
        id,
        SpaceActionImpact::CreatesResource,
        SpaceActionExecutionMode::Durable,
        ActionBinding {
            capability: execution_capability,
            purpose: routing_purpose,
        },
        input_roles,
        parameters_schema,
        output_contract,
    )
}

pub(super) const fn query_action(
    id: &'static str,
    execution_capability: Option<ExecutionCapability>,
    routing_purpose: Option<RoutingPurpose>,
    input_roles: &'static [SpaceActionInputRole],
    parameters_schema: ParameterSchemaFactory,
    output_contract: SpaceActionOutputContract,
) -> SpaceActionDefinition {
    definition(
        id,
        SpaceActionImpact::ReadOnly,
        SpaceActionExecutionMode::Immediate,
        ActionBinding {
            capability: execution_capability,
            purpose: routing_purpose,
        },
        input_roles,
        parameters_schema,
        output_contract,
    )
}

pub(super) const fn durable_query_action(
    id: &'static str,
    execution_capability: ExecutionCapability,
    routing_purpose: RoutingPurpose,
    input_roles: &'static [SpaceActionInputRole],
    parameters_schema: ParameterSchemaFactory,
    output_contract: SpaceActionOutputContract,
) -> SpaceActionDefinition {
    definition(
        id,
        SpaceActionImpact::ReadOnly,
        SpaceActionExecutionMode::Durable,
        ActionBinding {
            capability: Some(execution_capability),
            purpose: Some(routing_purpose),
        },
        input_roles,
        parameters_schema,
        output_contract,
    )
}

pub(super) const fn immediate_action(
    id: &'static str,
    input_roles: &'static [SpaceActionInputRole],
    parameters_schema: ParameterSchemaFactory,
    output_contract: SpaceActionOutputContract,
) -> SpaceActionDefinition {
    definition(
        id,
        SpaceActionImpact::CreatesResource,
        SpaceActionExecutionMode::Immediate,
        ActionBinding {
            capability: None,
            purpose: None,
        },
        input_roles,
        parameters_schema,
        output_contract,
    )
}

pub(super) const fn immediate_mutation_action(
    id: &'static str,
    parameters_schema: ParameterSchemaFactory,
    output_contract: SpaceActionOutputContract,
) -> SpaceActionDefinition {
    definition(
        id,
        SpaceActionImpact::MutatesResource,
        SpaceActionExecutionMode::Immediate,
        ActionBinding {
            capability: None,
            purpose: None,
        },
        &[],
        parameters_schema,
        output_contract,
    )
}

const fn definition(
    id: &'static str,
    impact: SpaceActionImpact,
    execution_mode: SpaceActionExecutionMode,
    binding: ActionBinding,
    input_roles: &'static [SpaceActionInputRole],
    parameters_schema: ParameterSchemaFactory,
    output_contract: SpaceActionOutputContract,
) -> SpaceActionDefinition {
    SpaceActionDefinition {
        id,
        version: 1,
        impact,
        execution_mode,
        approval_policy: if !matches!(impact, SpaceActionImpact::ReadOnly) {
            SpaceActionApprovalPolicy::ExactPlan
        } else {
            SpaceActionApprovalPolicy::TaskGrant
        },
        input_consistency: SpaceActionInputConsistency::PinnedRevision,
        execution_capability: binding.capability,
        routing_purpose: binding.purpose,
        input_roles,
        collection_input: None,
        parameters_schema,
        output_contract,
    }
}
