//! Display helpers to convert GraphQL service document AST type to GraphQL string.
//! Notes:
//! 1. When implementing Display trait for a **leaf level** Type, if it can go in a new-line,
//! write the `new-line` first **before** writing the content of the type.
//! (ex.) InputValueDefinition ouputs a newline **before** its contents.
//! 2. When implementing Display trait for an **enclosing / root type** Type, make sure it
//! outputs a `new-line` in the end **after** its contents.
//! (ex.) InputObjectType outputs a newline **after** its contents.
use super::*;
use crate::graphql_ast;
use std::fmt::{self, Write};

impl fmt::Display for SchemaDefinition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        ln_display_description_ln(f, &self.description)?;
        write!(f, "schema")?;
        display_directives(f, &self.directives)?;
        f.write_str(" {")?;
        self.query
            .as_ref()
            .map_or_else(|| Ok(()), |query_type| write!(f, "query: {}", query_type))?;
        self.mutation.as_ref().map_or_else(
            || Ok(()),
            |mutation_type| write!(f, "mutation :{}", mutation_type),
        )?;
        self.subscription.as_ref().map_or_else(
            || Ok(()),
            |subscription_type| write!(f, "subscription: {}", subscription_type),
        )?;
        f.write_str("\n}\n")
    }
}

impl fmt::Display for DirectiveDefinition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        ln_display_description_ln(f, &self.description)?;
        write!(f, "directive @{}", &self.name)?;
        if self.arguments.len() > 0 {
            write!(f, "(")?;
            self.arguments
                .iter()
                .try_for_each(|argument| f.write_str(&argument.to_string()))?;
            write!(f, "\n)")?;
        }
        if self.is_repeatable {
            write!(f, " repeatable")?;
        }
        write!(f, " on")?;
        self.locations
            .iter()
            .try_for_each(|loc| write!(f, "\n| {}", loc))?;
        write!(f, "\n")
    }
}

impl fmt::Display for DirectiveLocation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DirectiveLocation::Query => write!(f, "QUERY"),
            DirectiveLocation::Mutation => write!(f, "MUTATION"),
            DirectiveLocation::Subscription => write!(f, "SUBSCRIPTION"),
            DirectiveLocation::Field => write!(f, "FIELD"),
            DirectiveLocation::FragmentDefinition => write!(f, "FRAGMENT_DEFINITION"),
            DirectiveLocation::FragmentSpread => write!(f, "FRAGMENT_SPREAD"),
            DirectiveLocation::InlineFragment => write!(f, "INLINE_FRAGMENT"),
            DirectiveLocation::VariableDefinition => write!(f, "VARIABLE_DEFINITION"),
            DirectiveLocation::Schema => write!(f, "SCHEMA"),
            DirectiveLocation::Scalar => write!(f, "SCALAR"),
            DirectiveLocation::Object => write!(f, "OBJECT"),
            DirectiveLocation::FieldDefinition => write!(f, "FIELD_DEFINITION"),
            DirectiveLocation::ArgumentDefinition => write!(f, "ARGUMENT_DEFINITION"),
            DirectiveLocation::Interface => write!(f, "INTERFACE"),
            DirectiveLocation::Union => write!(f, "UNION"),
            DirectiveLocation::Enum => write!(f, "ENUM"),
            DirectiveLocation::EnumValue => write!(f, "ENUM_VALUE"),
            DirectiveLocation::InputObject => write!(f, "INPUT_OBJECT"),
            DirectiveLocation::InputFieldDefinition => write!(f, "INPUT_FIELD_DEFINITION"),
        }
    }
}

impl fmt::Display for TypeDefinition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        ln_display_description_ln(f, &self.description)?;
        match &self.kind {
            TypeKind::Scalar => {
                write!(f, "scalar {}", &self.name)?;
                display_directives(f, &self.directives)?;
            }
            TypeKind::Object(object_type) => {
                write!(f, "type {}", &self.name)?;
                display_implements(f, &object_type.implements)?;
                display_directives(f, &self.directives)?;
                display_type_inside_block(f, object_type)?;
            }
            TypeKind::Interface(interface_type) => {
                write!(f, "interface {}", &self.name)?;
                display_implements(f, &interface_type.implements)?;
                display_directives(f, &self.directives)?;
                display_type_inside_block(f, interface_type)?;
            }
            TypeKind::Union(union_type) => {
                write!(f, "union {}", &self.name)?;
                display_directives(f, &self.directives)?;
                write!(f, " = {}", union_type)?;
            }
            TypeKind::Enum(enum_type) => {
                write!(f, "enum {}", &self.name)?;
                display_directives(f, &self.directives)?;
                display_type_inside_block(f, enum_type)?;
            }
            TypeKind::InputObject(input_obj_type) => {
                write!(f, "input {}", &self.name)?;
                display_directives(f, &self.directives)?;
                display_type_inside_block(f, input_obj_type)?;
            }
        }
        write!(f, "\n") // Note: Output new-line at the end.
    }
}

impl fmt::Display for ObjectType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.fields
            .iter()
            .try_for_each(|field| f.write_str(&field.to_string()))
    }
}

impl fmt::Display for InterfaceType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.fields
            .iter()
            .try_for_each(|field| f.write_str(&field.to_string()))
    }
}

impl fmt::Display for FieldDefinition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        ln_display_description_ln(f, &self.description)?;
        if self.argments.len() == 0 {
            write!(f, "{}: {}", self.name, self.ty)?;
        } else {
            write!(f, "{}(", self.name)?;
            self.argments
                .iter()
                .try_for_each(|argument| f.write_str(&argument.to_string()))?;
            write!(f, "\n): {}", self.ty)?;
        }
        display_directives(f, &self.directives)
    }
}

impl fmt::Display for InputObjectType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.fields
            .iter()
            .try_for_each(|field| write!(f, "{}", field))
    }
}

impl fmt::Display for InputValueDefinition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Note: Start each input argument with a newline so that they needn't be comma separated.
        ln_display_description_ln(f, &self.description)?;
        write!(f, "{}: {}", self.name, self.ty)?;
        if self.default_value.is_some() {
            write!(f, " = {}", self.default_value.as_ref().unwrap())?;
        }
        self.directives
            .iter()
            .try_for_each(|directive| write!(f, " {}", directive))
    }
}

impl fmt::Display for UnionType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.members
            .iter()
            .try_for_each(|union_member| write!(f, "\n| {}", union_member))
    }
}

impl fmt::Display for EnumType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.values
            .iter()
            .try_for_each(|enum_value_def| f.write_str(&enum_value_def.to_string()))
    }
}

impl fmt::Display for EnumValueDefinition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        ln_display_description_ln(f, &self.description)?;
        f.write_str(&self.value.to_string())?;
        display_directives(f, &self.directives)
    }
}

impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.base.fmt(f)?;
        if !self.nullable {
            f.write_char('!')?;
        }
        Ok(())
    }
}

impl fmt::Display for BaseType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Named(name) => f.write_str(name),
            Self::List(ty) => write!(f, "[{}]", ty),
        }
    }
}

impl fmt::Display for ConstDirective {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let args: String = self
            .arguments
            .iter()
            .map(|(name, const_value)| format!("{name}: {const_value}, "))
            .collect();
        let args = args.trim_end_matches([',', ' ']);
        write!(f, "@{}({})", self.name, args)
    }
}

/// When there is a description, writes the description encloded by new-lines both before and after.
/// If the description is empty, then it just writes a new-line.
#[inline(always)]
fn ln_display_description_ln(f: &mut fmt::Formatter, desc: &Option<String>) -> fmt::Result {
    write!(f, "\n")?;
    if desc.is_some() {
        write!(f, "\"\"\"{}\"\"\"\n", desc.as_ref().unwrap())
    } else {
        Ok(())
    }
}

#[inline(always)]
fn display_directives(
    f: &mut fmt::Formatter,
    directives: &Vec<graphql_ast::ConstDirective>,
) -> fmt::Result {
    directives
        .iter()
        .try_for_each(|const_dir| write!(f, " {}", const_dir))
}

#[inline(always)]
fn display_implements(f: &mut fmt::Formatter, interfaces: &Vec<Name>) -> fmt::Result {
    if interfaces.len() > 1 {
        let interfaces_str = interfaces.iter().fold(" ".to_string(), |acc, interface| {
            format!("{}{} &", acc, interface)
        });
        let interfaces_str = interfaces_str.trim_end_matches(" &");
        write!(f, " implements{}", interfaces_str)
    } else {
        Ok(())
    }
}

#[inline(always)]
fn display_type_inside_block(f: &mut fmt::Formatter, r#type: impl fmt::Display) -> fmt::Result {
    f.write_str(" {")?;
    f.write_str(&r#type.to_string())?;
    f.write_str("\n}")
}

#[cfg(test)]
mod test {
    use graphql_value::{ConstValue, Name};

    use super::{ConstDirective, InputValueDefinition, Type};

    #[test]
    fn test_const_directive_display_trait() {
        let deprecated_directive = ConstDirective {
            name: Name::new("deprecated"),
            arguments: vec![(
                Name::new("reason"),
                ConstValue::String("Use some_other_field".to_string()),
            )],
        };

        assert_eq!(
            r#"@deprecated(reason: "Use some_other_field")"#,
            deprecated_directive.to_string()
        );

        let some_directive = ConstDirective {
            name: Name::new("some"),
            arguments: vec![
                (
                    Name::new("arg1"),
                    ConstValue::String("String value".to_string()),
                ),
                (Name::new("arg2"), ConstValue::Boolean(true)),
                (
                    Name::new("arg3"),
                    ConstValue::String("Another string value".to_string()),
                ),
            ],
        };

        assert_eq!(
            r#"@some(arg1: "String value", arg2: true, arg3: "Another string value")"#,
            some_directive.to_string()
        );
    }

    #[test]
    fn test_input_value_definition_display_trait() {
        let deprecated_directive = ConstDirective {
            name: Name::new("deprecated"),
            arguments: vec![(
                Name::new("reason"),
                ConstValue::String("Use some_other_field".to_string()),
            )],
        };

        let some_directive = ConstDirective {
            name: Name::new("some"),
            arguments: vec![
                (
                    Name::new("arg1"),
                    ConstValue::String("String value".to_string()),
                ),
                (Name::new("arg2"), ConstValue::Boolean(true)),
                (
                    Name::new("arg3"),
                    ConstValue::String("Another string value".to_string()),
                ),
            ],
        };

        let id_input_value = InputValueDefinition {
            description: Some("This is some id with default value as def_id".to_string()),
            name: Name::new("id"),
            ty: Type::new("ID").unwrap(),
            default_value: Some(ConstValue::String("def_id".to_string())),
            directives: vec![some_directive, deprecated_directive],
        };

        let id_input_value_expected_str = r#"
"""This is some id with default value as def_id"""
id: ID = "def_id" @some(arg1: "String value", arg2: true, arg3: "Another string value") @deprecated(reason: "Use some_other_field")"#;
        assert_eq!(id_input_value_expected_str, id_input_value.to_string());

        let id_input_value = InputValueDefinition {
            description: Some("This is some id with default value as def_id".to_string()),
            name: Name::new("id"),
            ty: Type::new("ID").unwrap(),
            default_value: None,
            directives: vec![],
        };

        let id_input_value_expected_str = r#"
"""This is some id with default value as def_id"""
id: ID"#;
        assert_eq!(id_input_value_expected_str, id_input_value.to_string());

        let id_input_value = InputValueDefinition {
            description: None,
            name: Name::new("id"),
            ty: Type::new("ID").unwrap(),
            default_value: None,
            directives: vec![],
        };

        let id_input_value_expected_str = r#"
id: ID"#;
        assert_eq!(id_input_value_expected_str, id_input_value.to_string());
    }

    #[test]
    fn test_input_object_type() {
        unimplemented!()
    }

    #[test]
    fn test_type_definition() {
        unimplemented!()
    }
}
