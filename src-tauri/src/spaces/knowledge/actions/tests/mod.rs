use std::collections::HashSet;

use super::definitions;

#[test]
fn registry_definitions_are_coherent() {
    let definitions = definitions();
    let ids = definitions
        .iter()
        .map(|definition| definition.id)
        .collect::<HashSet<_>>();

    assert_eq!(ids.len(), definitions.len());

    for definition in definitions {
        assert!(!definition.id.is_empty());
        assert!(definition.version > 0);
        assert!(!definition.input_roles.is_empty());
        assert!(!definition.output_contract.is_empty());
        assert_eq!(
            definition.execution_capability.is_some(),
            definition.routing_purpose.is_some(),
        );

        let parameter_keys = definition
            .parameters
            .iter()
            .map(|parameter| parameter.key)
            .collect::<HashSet<_>>();
        assert_eq!(parameter_keys.len(), definition.parameters.len());
    }
}
