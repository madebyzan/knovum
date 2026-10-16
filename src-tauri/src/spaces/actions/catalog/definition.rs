use crate::ai::contracts::{ExecutionCapability, RoutingPurpose};

use super::super::{
    ParameterSchemaFactory, SpaceActionDefinition, SpaceActionExecutionMode, SpaceActionImpact,
    SpaceActionInputConsistency, SpaceActionInputRole, SpaceActionOutputArity,
    SpaceActionOutputContract,
};

struct ActionBinding {
    capability: Option<ExecutionCapability>,
    purpose: Option<RoutingPurpose>,
}

pub(super) const fn action(
    action_id: &'static str,
    execution_capability: Option<ExecutionCapability>,
    routing_purpose: Option<RoutingPurpose>,
    input_roles: &'static [SpaceActionInputRole],
    parameters_schema: ParameterSchemaFactory,
    output_contract: SpaceActionOutputContract,
) -> SpaceActionDefinition {
    definition(
        action_id,
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
    action_id: &'static str,
    execution_capability: Option<ExecutionCapability>,
    routing_purpose: Option<RoutingPurpose>,
    input_roles: &'static [SpaceActionInputRole],
    parameters_schema: ParameterSchemaFactory,
    output_contract: SpaceActionOutputContract,
) -> SpaceActionDefinition {
    definition(
        action_id,
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
    action_id: &'static str,
    execution_capability: ExecutionCapability,
    routing_purpose: RoutingPurpose,
    input_roles: &'static [SpaceActionInputRole],
    parameters_schema: ParameterSchemaFactory,
    output_contract: SpaceActionOutputContract,
) -> SpaceActionDefinition {
    definition(
        action_id,
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
    action_id: &'static str,
    input_roles: &'static [SpaceActionInputRole],
    parameters_schema: ParameterSchemaFactory,
    output_contract: SpaceActionOutputContract,
) -> SpaceActionDefinition {
    definition(
        action_id,
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
    action_id: &'static str,
    parameters_schema: ParameterSchemaFactory,
    output_contract: SpaceActionOutputContract,
) -> SpaceActionDefinition {
    definition(
        action_id,
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
    action_id: &'static str,
    impact: SpaceActionImpact,
    execution_mode: SpaceActionExecutionMode,
    binding: ActionBinding,
    input_roles: &'static [SpaceActionInputRole],
    parameters_schema: ParameterSchemaFactory,
    output_contract: SpaceActionOutputContract,
) -> SpaceActionDefinition {
    SpaceActionDefinition {
        id: action_id,
        version: 1,
        impact,
        execution_mode,
        input_consistency: SpaceActionInputConsistency::PinnedRevision,
        execution_capability: binding.capability,
        routing_purpose: binding.purpose,
        input_roles,
        collection_input: None,
        parameters_schema,
        output_contract,
        output_arity: SpaceActionOutputArity::Single,
    }
}

/// Marks an Action whose input decides how many outputs it publishes.
pub(super) const fn discovered_outputs(
    mut definition: SpaceActionDefinition,
) -> SpaceActionDefinition {
    definition.output_arity = SpaceActionOutputArity::Discovered;
    definition
}
