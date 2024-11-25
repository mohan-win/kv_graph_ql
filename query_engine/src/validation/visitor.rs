use std::collections::HashMap;

use crate::graphql_parser::{
    types::{
        Directive, ExecutableDocument, Field, FragmentDefinition, OperationDefinition,
        Selection, SelectionSet, VariableDefinition,
    },
    Pos, Positioned,
};
use graphql_parser::types::{FragmentSpread, InlineFragment};
use graphql_value::Value;

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

pub(crate) trait Visitor<'a> {
    fn mode(&self) -> VisitMode {
        VisitMode::Normal
    }

    fn enter_document(
        &mut self,
        _ctx: &mut VisitorContext<'a>,
        _doc: &'a ExecutableDocument,
    ) {
    }
    fn exit_document(
        &mut self,
        _ctx: &mut VisitorContext<'a>,
        _doc: &'a ExecutableDocument,
    ) {
    }

    fn enter_operation_definition(
        &mut self,
        _ctx: &mut VisitorContext<'a>,
        _operation_definition: &'a Positioned<OperationDefinition>,
    ) {
    }
    fn exit_operation_definition(
        &mut self,
        _ctx: &mut VisitorContext<'a>,
        _operation_definition: &'a Positioned<OperationDefinition>,
    ) {
    }

    fn enter_fragment_definition(
        &mut self,
        _ctx: &mut VisitorContext<'a>,
        _name: &'a Name,
        _fragment_definition: &'a Positioned<FragmentDefinition>,
    ) {
    }
    fn exit_fragment_definition(
        &mut self,
        _ctx: &mut VisitorContext<'a>,
        _name: &'a Name,
        _fragment_definition: &'a Positioned<FragmentDefinition>,
    ) {
    }

    fn enter_variable_definition(
        &mut self,
        _ctx: &mut VisitorContext<'a>,
        _variable_definition: &'a Positioned<VariableDefinition>,
    ) {
    }
    fn exit_variable_definition(
        &mut self,
        _ctx: &mut VisitorContext<'a>,
        _variable_definition: &'a Positioned<VariableDefinition>,
    ) {
    }

    fn enter_directive(
        &mut self,
        _ctx: &mut VisitorContext<'a>,
        _directive: &'a Positioned<Directive>,
    ) {
    }
    fn exit_directive(
        &mut self,
        _ctx: &mut VisitorContext<'a>,
        _directive: &'a Positioned<Directive>,
    ) {
    }

    fn enter_argument(
        &mut self,
        _ctx: &'a VisitorContext<'a>,
        _name: &'a Positioned<Name>,
        _value: Positioned<Value>,
    ) {
    }
    fn exit_argument(
        &mut self,
        _ctx: &'a VisitorContext<'a>,
        _name: &'a Positioned<Name>,
        _value: Positioned<Value>,
    ) {
    }

    fn enter_selection_set(
        &mut self,
        _ctx: &mut VisitorContext<'a>,
        _selection_set: &'a Positioned<SelectionSet>,
    ) {
    }
    fn exit_selection_set(
        &mut self,
        _ctx: &mut VisitorContext<'a>,
        _selection_set: &'a Positioned<SelectionSet>,
    ) {
    }

    fn enter_selection(
        &mut self,
        _ctx: &mut VisitorContext<'a>,
        _selection: &'a Positioned<Selection>,
    ) {
    }
    fn exit_selection(
        &mut self,
        _ctx: &mut VisitorContext<'a>,
        _selection: &'a Positioned<Selection>,
    ) {
    }

    fn enter_field(
        &mut self,
        _ctx: &mut VisitorContext<'a>,
        _field: &'a Positioned<Field>,
    ) {
    }
    fn exit_field(
        &mut self,
        _ctx: &mut VisitorContext<'a>,
        _field: &'a Positioned<Field>,
    ) {
    }

    fn enter_fragment_spread(
        &mut self,
        _ctx: &mut VisitorContext<'a>,
        _fragment_spread: &'a Positioned<FragmentSpread>,
    ) {
    }
    fn exit_fragment_spread(
        &mut self,
        _ctx: &mut VisitorContext<'a>,
        _fragment_spread: &'a Positioned<FragmentSpread>,
    ) {
    }

    fn enter_inline_fragment(
        &mut self,
        _ctx: &mut VisitorContext<'a>,
        _inline_fragment: &'a Positioned<InlineFragment>,
    ) {
    }
    fn exit_inline_fragment(
        &mut self,
        _ctx: &mut VisitorContext<'a>,
        _inline_fragment: &'a Positioned<InlineFragment>,
    ) {
    }

    fn enter_input_value(
        &mut self,
        _ctx: &mut VisitorContext<'a>,
        _pos: Pos,
        _expected_type: &Option<MetaTypeName<'a>>,
        _value: &Value,
    ) {
    }
    fn exit_input_value(
        &mut self,
        _ctx: &mut VisitorContext<'a>,
        _pos: Pos,
        _expected_type: &Option<MetaTypeName<'a>>,
        _value: &Value,
    ) {
    }
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
