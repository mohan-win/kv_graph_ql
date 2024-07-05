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

impl fmt::Display for ServiceDocument {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.definitions
            .iter()
            .try_for_each(|definition| f.write_str(&definition.to_string()))
    }
}

impl fmt::Display for TypeSystemDefinition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TypeSystemDefinition::Schema(schema) => f.write_str(&schema.to_string()),
            TypeSystemDefinition::Type(r#type) => f.write_str(&r#type.to_string()),
            TypeSystemDefinition::Directive(directive) => f.write_str(&directive.to_string()),
        }
    }
}

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
        if self.arguments.len() == 0 {
            write!(f, "{}: {}", self.name, self.ty)?;
        } else {
            write!(f, "{}(", self.name)?;
            self.arguments
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
        if self.arguments.len() == 0 {
            write!(f, "@{}", self.name)
        } else {
            let args: String = self
                .arguments
                .iter()
                .map(|(name, const_value)| format!("{name}: {const_value}, "))
                .collect();
            let args = args.trim_end_matches([',', ' ']);
            write!(f, "@{}({})", self.name, args)
        }
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
            format!("{}{} & ", acc, interface)
        });
        let interfaces_str = interfaces_str.trim_end_matches(" & ");
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
    use std::vec;

    use super::*;
    use graphql_value::{ConstValue, Name};

    #[test]
    fn test_const_directive_def() {
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
    fn test_input_value_def() {
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
            description: Some("This is some id".to_string()),
            name: Name::new("id"),
            ty: Type::new("ID").unwrap(),
            default_value: None,
            directives: vec![],
        };

        let id_input_value_expected_str = r#"
"""This is some id"""
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
    fn test_input_object_def() {
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

        let id_input_value1 = InputValueDefinition {
            description: Some("This is some id with default value as def_id".to_string()),
            name: Name::new("id1"),
            ty: Type::new("ID").unwrap(),
            default_value: Some(ConstValue::String("def_id".to_string())),
            directives: vec![some_directive, deprecated_directive],
        };

        let id_input_value2 = InputValueDefinition {
            description: Some("This is some id".to_string()),
            name: Name::new("id2"),
            ty: Type::new("ID").unwrap(),
            default_value: None,
            directives: vec![],
        };

        let id_input_value3 = InputValueDefinition {
            description: None,
            name: Name::new("id3"),
            ty: Type::new("ID").unwrap(),
            default_value: None,
            directives: vec![],
        };
        let fields = vec![id_input_value1, id_input_value2, id_input_value3];
        let input_object_type = InputObjectType { fields };
        let expected_input_object_type_graphql = r#"
"""This is some id with default value as def_id"""
id1: ID = "def_id" @some(arg1: "String value", arg2: true, arg3: "Another string value") @deprecated(reason: "Use some_other_field")
"""This is some id"""
id2: ID
id3: ID"#;
        assert_eq!(
            expected_input_object_type_graphql,
            input_object_type.to_string()
        );
    }

    #[test]
    fn test_type_definition_input() {
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

        let id_input_value1 = InputValueDefinition {
            description: Some("This is some id with default value as def_id".to_string()),
            name: Name::new("id1"),
            ty: Type::new("ID").unwrap(),
            default_value: Some(ConstValue::String("def_id".to_string())),
            directives: vec![some_directive.clone(), deprecated_directive],
        };

        let id_input_value2 = InputValueDefinition {
            description: Some("This is some id".to_string()),
            name: Name::new("id2"),
            ty: Type::new("ID").unwrap(),
            default_value: None,
            directives: vec![],
        };

        let id_input_value3 = InputValueDefinition {
            description: None,
            name: Name::new("id3"),
            ty: Type::new("ID").unwrap(),
            default_value: None,
            directives: vec![],
        };
        let fields = vec![id_input_value1, id_input_value2, id_input_value3];
        let input_object_type = InputObjectType { fields };
        let input_type_def = TypeDefinition {
            extend: false,
            description: Some("Input object type definition with 3 fields".to_string()),
            name: Name::new("MyInputObject"),
            directives: vec![some_directive.clone(), some_directive],
            kind: TypeKind::InputObject(input_object_type),
        };
        let expected_input_type_def_graphql = r#"
"""Input object type definition with 3 fields"""
input MyInputObject @some(arg1: "String value", arg2: true, arg3: "Another string value") @some(arg1: "String value", arg2: true, arg3: "Another string value") {
"""This is some id with default value as def_id"""
id1: ID = "def_id" @some(arg1: "String value", arg2: true, arg3: "Another string value") @deprecated(reason: "Use some_other_field")
"""This is some id"""
id2: ID
id3: ID
}
"#;
        assert_eq!(expected_input_type_def_graphql, input_type_def.to_string());
    }

    #[test]
    fn test_object_def() {
        let expected_graphql = r#"
"""Root query object"""
type Query {
"""Fetch users for the given criteria"""
users(
"""Users where input"""
where: UserWhereInput
orderBy: UserOrderByInput
skip: Int
after: ID
before: ID
first: Int
last: Int
): [User!]! @deprecated(reason: "use userConnection for better performance")
adminUser: User
me: User!
}
"#;
        let users_field_args = vec![
            InputValueDefinition {
                description: Some("Users where input".to_string()),
                name: Name::new("where"),
                ty: Type::new("UserWhereInput").unwrap(),
                default_value: None,
                directives: vec![],
            },
            InputValueDefinition {
                description: None,
                name: Name::new("orderBy"),
                ty: Type::new("UserOrderByInput").unwrap(),
                default_value: None,
                directives: vec![],
            },
            InputValueDefinition {
                description: None,
                name: Name::new("skip"),
                ty: Type::new("Int").unwrap(),
                default_value: None,
                directives: vec![],
            },
            InputValueDefinition {
                description: None,
                name: Name::new("after"),
                ty: Type::new("ID").unwrap(),
                default_value: None,
                directives: vec![],
            },
            InputValueDefinition {
                description: None,
                name: Name::new("before"),
                ty: Type::new("ID").unwrap(),
                default_value: None,
                directives: vec![],
            },
            InputValueDefinition {
                description: None,
                name: Name::new("first"),
                ty: Type::new("Int").unwrap(),
                default_value: None,
                directives: vec![],
            },
            InputValueDefinition {
                description: None,
                name: Name::new("last"),
                ty: Type::new("Int").unwrap(),
                default_value: None,
                directives: vec![],
            },
        ];
        let deprecated_directive = ConstDirective {
            name: Name::new("deprecated"),
            arguments: vec![(
                Name::new("reason"),
                ConstValue::String("use userConnection for better performance".to_string()),
            )],
        };
        let users_field = FieldDefinition {
            description: Some("Fetch users for the given criteria".to_string()),
            name: Name::new("users"),
            arguments: users_field_args,
            ty: Type::new("[User!]!").unwrap(),
            directives: vec![deprecated_directive],
        };
        let admin_field = FieldDefinition {
            description: None,
            name: Name::new("adminUser"),
            arguments: vec![],
            ty: Type::new("User").unwrap(),
            directives: vec![],
        };
        let me_field = FieldDefinition {
            description: None,
            name: Name::new("me"),
            arguments: vec![],
            ty: Type::new("User!").unwrap(),
            directives: vec![],
        };

        let query_object_type = ObjectType {
            implements: vec![],
            fields: vec![users_field, admin_field, me_field],
        };

        let query_type_def = TypeDefinition {
            extend: false,
            description: Some("Root query object".to_string()),
            name: Name::new("Query"),
            directives: vec![],
            kind: TypeKind::Object(query_object_type),
        };
        assert_eq!(expected_graphql, query_type_def.to_string());
    }
    #[test]
    fn test_object_def_1() {
        let expected_graphql = r#"
"""User system model"""
type User implements Node & Entity @deprecated(reason: "use UserV2 when saving new users") {
"""The unique identifier"""
id: ID! @unique
email: String! @unique
"""The time the document was created"""
createdAt: DateTime!
}
"#;
        let deprecated_user_directive = ConstDirective {
            name: Name::new("deprecated"),
            arguments: vec![(
                Name::new("reason"),
                ConstValue::String("use UserV2 when saving new users".to_string()),
            )],
        };
        let unique_directive = ConstDirective {
            name: Name::new("unique"),
            arguments: vec![],
        };
        let object_type_def = TypeDefinition {
            extend: false,
            description: Some("User system model".to_string()),
            name: Name::new("User"),
            directives: vec![deprecated_user_directive],
            kind: TypeKind::Object(ObjectType {
                implements: vec![Name::new("Node"), Name::new("Entity")],
                fields: vec![
                    FieldDefinition {
                        description: Some("The unique identifier".to_string()),
                        name: Name::new("id"),
                        arguments: vec![],
                        ty: Type::new("ID!").unwrap(),
                        directives: vec![unique_directive.clone()],
                    },
                    FieldDefinition {
                        description: None,
                        name: Name::new("email"),
                        arguments: vec![],
                        ty: Type::new("String!").unwrap(),
                        directives: vec![unique_directive.clone()],
                    },
                    FieldDefinition {
                        description: Some("The time the document was created".to_string()),
                        name: Name::new("createdAt"),
                        arguments: vec![],
                        ty: Type::new("DateTime!").unwrap(),
                        directives: vec![],
                    },
                ],
            }),
        };
        assert_eq!(expected_graphql, object_type_def.to_string());
    }

    #[test]
    fn test_interface_def() {
        let expected_graphql = r#"
"""Root query interface"""
interface RootQuery {
"""Fetch users for the given criteria"""
users(
"""Users where input"""
where: UserWhereInput
orderBy: UserOrderByInput
skip: Int
after: ID
before: ID
first: Int
last: Int
): [User!]! @deprecated(reason: "use userConnection for better performance")
adminUser: User
me: User!
}
"#;
        let users_field_args = vec![
            InputValueDefinition {
                description: Some("Users where input".to_string()),
                name: Name::new("where"),
                ty: Type::new("UserWhereInput").unwrap(),
                default_value: None,
                directives: vec![],
            },
            InputValueDefinition {
                description: None,
                name: Name::new("orderBy"),
                ty: Type::new("UserOrderByInput").unwrap(),
                default_value: None,
                directives: vec![],
            },
            InputValueDefinition {
                description: None,
                name: Name::new("skip"),
                ty: Type::new("Int").unwrap(),
                default_value: None,
                directives: vec![],
            },
            InputValueDefinition {
                description: None,
                name: Name::new("after"),
                ty: Type::new("ID").unwrap(),
                default_value: None,
                directives: vec![],
            },
            InputValueDefinition {
                description: None,
                name: Name::new("before"),
                ty: Type::new("ID").unwrap(),
                default_value: None,
                directives: vec![],
            },
            InputValueDefinition {
                description: None,
                name: Name::new("first"),
                ty: Type::new("Int").unwrap(),
                default_value: None,
                directives: vec![],
            },
            InputValueDefinition {
                description: None,
                name: Name::new("last"),
                ty: Type::new("Int").unwrap(),
                default_value: None,
                directives: vec![],
            },
        ];
        let deprecated_directive = ConstDirective {
            name: Name::new("deprecated"),
            arguments: vec![(
                Name::new("reason"),
                ConstValue::String("use userConnection for better performance".to_string()),
            )],
        };
        let users_field = FieldDefinition {
            description: Some("Fetch users for the given criteria".to_string()),
            name: Name::new("users"),
            arguments: users_field_args,
            ty: Type::new("[User!]!").unwrap(),
            directives: vec![deprecated_directive],
        };
        let admin_field = FieldDefinition {
            description: None,
            name: Name::new("adminUser"),
            arguments: vec![],
            ty: Type::new("User").unwrap(),
            directives: vec![],
        };
        let me_field = FieldDefinition {
            description: None,
            name: Name::new("me"),
            arguments: vec![],
            ty: Type::new("User!").unwrap(),
            directives: vec![],
        };

        let query_interface_type = InterfaceType {
            implements: vec![],
            fields: vec![users_field, admin_field, me_field],
        };

        let query_interface_def = TypeDefinition {
            extend: false,
            description: Some("Root query interface".to_string()),
            name: Name::new("RootQuery"),
            directives: vec![],
            kind: TypeKind::Interface(query_interface_type),
        };
        assert_eq!(expected_graphql, query_interface_def.to_string());
    }
    #[test]
    fn test_interface_def_1() {
        let expected_graphql = r#"
"""User system model interface"""
interface User implements Node & Entity @deprecated(reason: "use UserV2 when saving new users") {
"""The unique identifier"""
id: ID! @unique
email: String! @unique
"""The time the document was created"""
createdAt: DateTime!
}
"#;
        let deprecated_user_directive = ConstDirective {
            name: Name::new("deprecated"),
            arguments: vec![(
                Name::new("reason"),
                ConstValue::String("use UserV2 when saving new users".to_string()),
            )],
        };
        let unique_directive = ConstDirective {
            name: Name::new("unique"),
            arguments: vec![],
        };
        let interface_type_def = TypeDefinition {
            extend: false,
            description: Some("User system model interface".to_string()),
            name: Name::new("User"),
            directives: vec![deprecated_user_directive],
            kind: TypeKind::Interface(InterfaceType {
                implements: vec![Name::new("Node"), Name::new("Entity")],
                fields: vec![
                    FieldDefinition {
                        description: Some("The unique identifier".to_string()),
                        name: Name::new("id"),
                        arguments: vec![],
                        ty: Type::new("ID!").unwrap(),
                        directives: vec![unique_directive.clone()],
                    },
                    FieldDefinition {
                        description: None,
                        name: Name::new("email"),
                        arguments: vec![],
                        ty: Type::new("String!").unwrap(),
                        directives: vec![unique_directive.clone()],
                    },
                    FieldDefinition {
                        description: Some("The time the document was created".to_string()),
                        name: Name::new("createdAt"),
                        arguments: vec![],
                        ty: Type::new("DateTime!").unwrap(),
                        directives: vec![],
                    },
                ],
            }),
        };
        assert_eq!(expected_graphql, interface_type_def.to_string());
    }

    #[test]
    fn test_union_type_def() {
        let expected_graphql = r#"
"""Search result union"""
union SearchResult = 
| User
| Post
"#;
        let expected_graphql_1 = r#"
union Person @nightly(reason: "This is not yet stable") = 
| User
| Admin
| Guest
"#;
        let search_result_union = TypeDefinition {
            extend: false,
            description: Some("Search result union".to_string()),
            name: Name::new("SearchResult"),
            directives: vec![],
            kind: TypeKind::Union(UnionType {
                members: vec![Name::new("User"), Name::new("Post")],
            }),
        };

        assert_eq!(expected_graphql, search_result_union.to_string());

        let person_union = TypeDefinition {
            extend: false,
            description: None,
            name: Name::new("Person"),
            directives: vec![ConstDirective {
                name: Name::new("nightly"),
                arguments: vec![(
                    Name::new("reason"),
                    ConstValue::String("This is not yet stable".to_string()),
                )],
            }],
            kind: TypeKind::Union(UnionType {
                members: vec![Name::new("User"), Name::new("Admin"), Name::new("Guest")],
            }),
        };

        assert_eq!(expected_graphql_1, person_union.to_string());
    }

    #[test]
    fn test_enum_type() {
        let expected_graphql = r#"
"""User role enum"""
enum Role @saveAsNumberInDB(startsFrom: 0, maxValue: 255) {
"""The User"""
USER
"""The Admin"""
ADMIN
"""The old root role"""
ROOT @deprecated(reason: "Use either USER or ADMIN")
}
"#;
        let expected_graphql_1 = r#"
enum LivingThings {
ANIMAL
PLANT
OTHER
}
"#;
        let user_role_enum = TypeDefinition {
            extend: false,
            description: Some("User role enum".to_string()),
            name: Name::new("Role"),
            directives: vec![ConstDirective {
                name: Name::new("saveAsNumberInDB"),
                arguments: vec![
                    (Name::new("startsFrom"), 0.into()),
                    (Name::new("maxValue"), 255.into()),
                ],
            }],
            kind: TypeKind::Enum(EnumType {
                values: vec![
                    EnumValueDefinition {
                        description: Some("The User".to_string()),
                        value: Name::new("USER"),
                        directives: vec![],
                    },
                    EnumValueDefinition {
                        description: Some("The Admin".to_string()),
                        value: Name::new("ADMIN"),
                        directives: vec![],
                    },
                    EnumValueDefinition {
                        description: Some("The old root role".to_string()),
                        value: Name::new("ROOT"),
                        directives: vec![ConstDirective {
                            name: Name::new("deprecated"),
                            arguments: vec![(
                                Name::new("reason"),
                                ConstValue::String("Use either USER or ADMIN".to_string()),
                            )],
                        }],
                    },
                ],
            }),
        };
        assert_eq!(expected_graphql, user_role_enum.to_string());

        let living_things_enum = TypeDefinition {
            extend: false,
            description: None,
            name: Name::new("LivingThings"),
            directives: vec![],
            kind: TypeKind::Enum(EnumType {
                values: vec![
                    EnumValueDefinition {
                        description: None,
                        value: Name::new("ANIMAL"),
                        directives: vec![],
                    },
                    EnumValueDefinition {
                        description: None,
                        value: Name::new("PLANT"),
                        directives: vec![],
                    },
                    EnumValueDefinition {
                        description: None,
                        value: Name::new("OTHER"),
                        directives: vec![],
                    },
                ],
            }),
        };
        assert_eq!(expected_graphql_1, living_things_enum.to_string());
    }

    #[test]
    fn test_scalar_type_def() {
        let expected_graphql = r#"
"""Date scalar"""
scalar Date @deprecated(reason: "Use DateTime scalar")
"#;
        let expected_graphql_1 = r#"
scalar DateTime @auto_generated @version(no: 1)
"#;
        let date_scalar = TypeDefinition {
            extend: false,
            description: Some("Date scalar".to_string()),
            name: Name::new("Date"),
            directives: vec![ConstDirective {
                name: Name::new("deprecated"),
                arguments: vec![(
                    Name::new("reason"),
                    ConstValue::String("Use DateTime scalar".to_string()),
                )],
            }],
            kind: TypeKind::Scalar,
        };

        assert_eq!(expected_graphql, date_scalar.to_string());

        let date_time_scalar = TypeDefinition {
            extend: false,
            description: None,
            name: Name::new("DateTime"),
            directives: vec![
                ConstDirective {
                    name: Name::new("auto_generated"),
                    arguments: vec![],
                },
                ConstDirective {
                    name: Name::new("version"),
                    arguments: vec![(Name::new("no"), 1.into())],
                },
            ],
            kind: TypeKind::Scalar,
        };

        assert_eq!(expected_graphql_1, date_time_scalar.to_string());
    }
}
