use std::collections::HashMap;

use crate::graphql_parser::{
    types::{ExecutableDocument, Field, FragmentDefinition, VariableDefinition},
    Pos, Positioned,
};

use crate::{
    error::{ServerError, ServerResult},
    registry::{self, MetaTypeName},
    InputType, Name, Variables,
};

pub struct VisitorContext<'a> {
    pub(crate) registry: &'a registry::Registry,
    pub(crate) variables: Option<&'a Variables>,
    pub(crate) errors: Vec<RuleError>,
    type_stack: Vec<Option<&'a registry::MetaType>>,
    input_type: Vec<Option<MetaTypeName<'a>>>,
    fragments: &'a HashMap<Name, Positioned<FragmentDefinition>>,
}

impl<'a> VisitorContext<'a> {
    pub(crate) fn new(
        registry: &'a registry::Registry,
        doc: &'a ExecutableDocument,
        variables: Option<&'a Variables>,
    ) -> Self {
        Self {
            registry,
            variables,
            errors: Default::default(),
            type_stack: Default::default(),
            input_type: Default::default(),
            fragments: &doc.fragments,
        }
    }

    pub(crate) fn report_error<T: Into<String>>(&mut self, locations: Vec<Pos>, msg: T) {
        self.errors.push(RuleError::new(locations, msg));
    }

    pub(crate) fn append_errors(&mut self, errors: Vec<RuleError>) {
        self.errors.extend(errors);
    }

    pub(crate) fn with_type<F: FnMut(&mut VisitorContext<'a>)>(
        &mut self,
        ty: Option<&'a registry::MetaType>,
        mut f: F,
    ) {
        self.type_stack.push(ty);
        f(self);
        self.type_stack.pop();
    }

    pub(crate) fn with_input_type<F: FnMut(&mut VisitorContext<'a>)>(
        &mut self,
        ty: Option<MetaTypeName<'a>>,
        mut f: F,
    ) {
        self.input_type.push(ty);
        f(self);
        self.input_type.pop();
    }

    pub(crate) fn parent_type(&self) -> Option<&'a registry::MetaType> {
        if self.type_stack.len() >= 2 {
            self.type_stack
                .get(self.type_stack.len() - 2)
                .copied()
                .flatten()
        } else {
            None
        }
    }

    pub(crate) fn current_type(&self) -> Option<&'a registry::MetaType> {
        self.type_stack.last().copied().flatten()
    }

    pub(crate) fn is_known_fragment(&self, name: &str) -> bool {
        self.fragments.contains_key(name)
    }

    pub(crate) fn fragment(
        &self,
        name: &str,
    ) -> Option<&'a Positioned<FragmentDefinition>> {
        self.fragments.get(name)
    }

    #[doc(hidden)]
    pub fn param_value<T: InputType>(
        &self,
        variable_definitions: &[Positioned<VariableDefinition>],
        field: &Field,
        name: &str,
        default: Option<fn() -> T>,
    ) -> ServerResult<T> {
        let value = field.get_argument(name).cloned();

        if value.is_none() {
            if let Some(default) = default {
                return Ok(default());
            }
        }

        let (pos, value) = match value {
            Some(value) => (
                value.pos,
                Some(value.node.into_const_with(|name| {
                    variable_definitions
                        .iter()
                        .find(|def| def.node.name.node == name)
                        .and_then(|def| {
                            if let Some(variables) = self.variables {
                                variables
                                    .get(&def.node.name.node)
                                    .or_else(|| def.node.default_value())
                            } else {
                                None
                            }
                        })
                        .cloned()
                        .ok_or_else(|| {
                            ServerError::new(
                                format!("Variable {} is not defined", name),
                                Some(value.pos),
                            )
                        })
                })?),
            ),
            None => (Pos::default(), None),
        };

        T::parse(value).map_err(|e| e.into_server_error(pos))
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum VisitMode {
    Normal,
    Inline,
}

#[derive(Debug, PartialEq)]
pub(crate) struct RuleError {
    pub(crate) locations: Vec<Pos>,
    pub(crate) message: String,
}

impl RuleError {
    pub(crate) fn new(locations: Vec<Pos>, msg: impl Into<String>) -> Self {
        RuleError {
            locations,
            message: msg.into(),
        }
    }
}
