use std::ops::Div;

use crate::ast::{
    AttribArg, Attribute, ConfigDecl, ConfigPair, DataModel, Declaration, EnumDecl, FieldDecl,
    FieldType, ModelDecl, PrimitiveType, Token, Type,
};
use chumsky::text::{self, ascii};
use chumsky::{extra::Err, prelude::*};

pub mod semantic_analysis;
pub use semantic_analysis::SemanticError;

pub fn new<'src>() -> impl Parser<'src, &'src str, DataModel<'src>, Err<Rich<'src, char>>> {
    delcarations().try_map(|decls, span| {
        semantic_analysis::to_data_model(decls, true).map_or_else(
            |errs| Err(Rich::custom(span, format!("{errs:#?}"))),
            |ast| match semantic_analysis::semantic_update(&ast) {
                Err(errs) => {
                    println!("todo:: remove errs = {errs:#?}");
                    Err(Rich::custom(span, format!("{errs:#?}")))
                }
                Ok(()) => Ok(ast),
            },
        )
    })
}

#[inline(always)]
pub(crate) fn delcarations<'src>(
) -> impl Parser<'src, &'src str, Vec<Declaration<'src>>, Err<Rich<'src, char>>> {
    config_decl()
        .or(enum_decl())
        .or(model_decl())
        .repeated()
        .collect::<Vec<Declaration>>()
}

#[inline(always)]
fn string<'src>() -> impl Parser<'src, &'src str, Token<'src>, Err<Rich<'src, char>>> {
    just('"')
        .then(any().filter(|c: &char| *c != '"').repeated())
        .then(just('"'))
        .to_slice()
        .map_with(|str, e| Token::Str(str, e.span()))
}

#[inline(always)]
fn number<'src>() -> impl Parser<'src, &'src str, Token<'src>, Err<Rich<'src, char>>> {
    text::int(10)
        .then(just('.').then(text::int(10)).or_not())
        .map_with(|(int_part, fract_part): (&str, Option<(char, &str)>), e| {
            let i: i64 = int_part.parse().unwrap();
            if fract_part.is_none() {
                Token::Int(i, e.span())
            } else {
                let number_of_digits = fract_part.unwrap().1.chars().count() as u32; // IMPORTANT: Note that we are counting characters, not counting grapheme clusters. In this use case it's ok.
                let thousands = (10 as i64).pow(number_of_digits) as f64;
                let fract_part = fract_part.unwrap().1.parse::<i64>().unwrap() as f64;
                Token::Float(i as f64 + fract_part.div(thousands), e.span())
            }
        })
}

#[inline(always)]
fn bool<'src>() -> impl Parser<'src, &'src str, Token<'src>, Err<Rich<'src, char>>> {
    text::keyword("true")
        .or(text::keyword("false"))
        .to_slice()
        .map_with(|b: &str, e| Token::Bool(b.parse().unwrap(), e.span()))
}

#[inline(always)]
fn config_pair<'src>() -> impl Parser<'src, &'src str, ConfigPair<'src>, Err<Rich<'src, char>>> {
    ascii::ident()
        .map_with(|ident, e| Token::Ident(ident, e.span()))
        .padded()
        .then(just('=').padded())
        .then(bool().or(number()).or(string()).padded())
        .map(
            |((key, _), value): ((Token<'src>, char), Token<'src>)| ConfigPair {
                name: key,
                value: value.try_into().unwrap(),
            },
        )
}

#[inline(always)]
fn config_decl<'src>() -> impl Parser<'src, &'src str, Declaration<'src>, Err<Rich<'src, char>>> {
    text::keyword("config")
        .padded()
        .then(
            ascii::ident()
                .map_with(|tok, e| Token::Ident(tok, e.span()))
                .padded(),
        )
        .then(just('{'))
        .then(config_pair().repeated().collect::<Vec<ConfigPair>>())
        .then(just('}').padded())
        .map(|((((_, config_name), _), config_pairs), _)| {
            Declaration::Config(ConfigDecl {
                name: config_name,
                config_pairs,
            })
        })
}

#[inline(always)]
fn enum_decl<'src>() -> impl Parser<'src, &'src str, Declaration<'src>, Err<Rich<'src, char>>> {
    let identifier = ascii::ident().map_with(|tok, e| Token::Ident(tok, e.span()));
    text::keyword("enum")
        .padded()
        .then(identifier.padded())
        .then(just('{'))
        .then(
            identifier
                .padded()
                .repeated()
                .at_least(1)
                .collect::<Vec<Token>>(),
        )
        .then(just('}').padded())
        .map(
            |((((_, enum_name), _open_brace), enum_elements), _close_brace)| {
                Declaration::Enum(EnumDecl {
                    name: enum_name,
                    elements: enum_elements,
                })
            },
        )
}

#[inline(always)]
fn field_type<'src>() -> impl Parser<'src, &'src str, FieldType<'src>, Err<Rich<'src, char>>> {
    let primitive_type = text::keyword("ShortStr")
        .or(text::keyword("LongStr"))
        .or(text::keyword("DateTime"))
        .or(text::keyword("Int32"))
        .or(text::keyword("Int64"))
        .or(text::keyword("Float"));

    primitive_type
        .or(text::ascii::ident())
        .then(just("?").or(just("[]")).or_not())
        .map_with(|(r#type, optional_or_array), e| {
            let r#type = match r#type {
                "ShortStr" => Some(PrimitiveType::ShortStr),
                "LongStr" => Some(PrimitiveType::LongStr),
                "DateTime" => Some(PrimitiveType::DateTime),
                "Int32" => Some(PrimitiveType::Int32),
                "Int64" => Some(PrimitiveType::Int64),
                "Float64" => Some(PrimitiveType::Float64),
                _ => None,
            }
            .map_or(
                Type::Unknown(Token::Ident(r#type, e.span())),
                Type::Primitive,
            );
            FieldType::new(
                r#type,
                optional_or_array.map_or(false, |v| v.eq("?")),
                optional_or_array.map_or(false, |v| v.eq("[]")),
            )
        })
}

#[inline(always)]
fn attribute<'src>() -> impl Parser<'src, &'src str, Attribute<'src>, Err<Rich<'src, char>>> {
    let attribute_arg = ascii::ident().then(just("()").or_not());
    just('@')
        .then(ascii::ident())
        .then(just('(').then(attribute_arg).then(just(')')).or_not())
        .map_with(|((_at, name), arg), e| {
            let arg = arg.map(|((_open_paran, (arg, parans)), _close_paran)| AttribArg {
                name: Token::Ident(arg, e.span()),
                is_function: parans.is_some(),
            });
            Attribute {
                name: Token::Ident(name, e.span()),
                arg,
            }
        })
}

#[inline(always)]
fn field_decl<'src>() -> impl Parser<'src, &'src str, FieldDecl<'src>, Err<Rich<'src, char>>> {
    ascii::ident()
        .padded()
        .map_with(|tok, e| Token::Ident(tok, e.span()))
        .then(field_type().padded())
        .then(attribute().padded().repeated().collect::<Vec<Attribute>>())
        .map(|((name, field_type), attributes)| FieldDecl {
            name,
            field_type,
            attributes,
        })
}

#[inline(always)]
fn model_decl<'src>() -> impl Parser<'src, &'src str, Declaration<'src>, Err<Rich<'src, char>>> {
    text::keyword("model")
        .padded()
        .then(ascii::ident().padded())
        .then(just('{'))
        .then(field_decl().repeated().collect::<Vec<FieldDecl>>())
        .then(just('}').padded())
        .map_with(
            |((((_model, name), _open_brace), fields), _close_brace), e| {
                Declaration::Model(ModelDecl {
                    name: Token::Ident(name, e.span()),
                    fields,
                })
            },
        )
}

#[cfg(test)]
mod tests {
    use std::{collections::HashMap, vec};

    use crate::ast::{ConfigValue, Span};

    use super::*;

    #[test]
    fn test_string() {
        assert_eq!(
            string().parse("\"Valid String\"").into_result(),
            Ok(Token::Str("\"Valid String\"", Span::new(0, 0)))
        );
        assert!(string()
            .parse(" \"Invalid string because whitespaces\" ")
            .into_result()
            .is_err());
        assert!(string()
            .parse("Invalid string because no quotes")
            .into_result()
            .is_err());
        assert!(string()
            .parse("\"Invalid string because no end quotes")
            .into_result()
            .is_err());
    }

    #[test]
    fn test_number() {
        assert_eq!(
            number().parse("123456789987654321").into_result(),
            Ok(Token::Int(123456789987654321, Span::new(0, 0)))
        );
        assert!(number()
            .parse(" 123456789987654321 ")
            .into_result()
            .is_err(),);
        assert!(number().parse("1A").into_result().is_err(),);

        assert_eq!(
            number().parse("12345678.9987654321").into_result(),
            Ok(Token::Float(12345678.9987654321, Span::new(0, 0)))
        );
        assert!(number().parse("12345678.").into_result().is_err(),);
        assert!(number().parse("12345678.as").into_result().is_err(),);
        assert!(number().parse("1A.123").into_result().is_err(),);
        assert!(number().parse(" 12345678.123 ").into_result().is_err(),);
    }

    #[test]
    fn test_bool() {
        assert_eq!(
            bool().parse("true").into_result(),
            Ok(Token::Bool(true, Span::new(0, 0)))
        );
        assert_eq!(
            bool().parse("false").into_result(),
            Ok(Token::Bool(false, Span::new(0, 0)))
        );
        assert!(bool().parse("True").into_result().is_err());
        assert!(bool().parse("False").into_result().is_err());
    }

    #[test]
    fn test_config_pair() {
        assert_eq!(
            config_pair()
                .parse("provider=\"foundationDB\"")
                .into_result(),
            Ok(ConfigPair {
                name: Token::Ident("provider", Span::new(0, 0)),
                value: ConfigValue::Str("\"foundationDB\"", Span::new(0, 0))
            })
        );

        assert_eq!(
            config_pair()
                .parse("\n   provider   =   \"foundationDB\"  \n")
                .into_result(),
            Ok(ConfigPair {
                name: Token::Ident("provider", Span::new(0, 0)),
                value: ConfigValue::Str("\"foundationDB\"", Span::new(0, 0))
            })
        );

        assert_eq!(
            config_pair()
                .parse("\n   is_local   =   true  \n")
                .into_result(),
            Ok(ConfigPair {
                name: Token::Ident("is_local", Span::new(0, 0)),
                value: ConfigValue::Bool(true, Span::new(0, 0))
            })
        );

        assert_eq!(
            config_pair()
                .parse("\n   port   =   1233  \n")
                .into_result(),
            Ok(ConfigPair {
                name: Token::Ident("port", Span::new(0, 0)),
                value: ConfigValue::Int(1233, Span::new(0, 0))
            })
        );

        assert_eq!(
            config_pair()
                .parse("\n   time_out_in_secs   =   10.25  \n")
                .into_result(),
            Ok(ConfigPair {
                name: Token::Ident("time_out_in_secs", Span::new(0, 0)),
                value: ConfigValue::Float(10.25, Span::new(0, 0))
            })
        );

        assert!(config_pair()
            .parse("\n   provider   =   foundationDB  \n")
            .into_result()
            .is_err());
        assert!(config_pair()
            .parse("\n   provider =\n")
            .into_result()
            .is_err());
        assert!(config_pair()
            .parse("\n   provider   foundationDB  \n")
            .into_result()
            .is_err())
    }

    #[test]
    fn test_config_decl() {
        let config_str = r#" 
            config db {
                provider = "foundationDB"
                port = 1233
                time_out_in_secs = 12.10
            }
        "#;

        assert_eq!(
            config_decl().parse(config_str).into_result(),
            Ok(Declaration::Config(ConfigDecl {
                name: Token::Ident("db", Span::new(0, 0)),
                config_pairs: vec![
                    ConfigPair {
                        name: Token::Ident("provider", Span::new(0, 0)),
                        value: ConfigValue::Str("\"foundationDB\"", Span::new(0, 0))
                    },
                    ConfigPair {
                        name: Token::Ident("port", Span::new(0, 0)),
                        value: ConfigValue::Int(1233, Span::new(0, 0))
                    },
                    ConfigPair {
                        name: Token::Ident("time_out_in_secs", Span::new(0, 0)),
                        value: ConfigValue::Float(12.10, Span::new(0, 0))
                    }
                ]
            }))
        );
    }

    #[test]
    fn test_enum_decl() {
        let enum_str = r#" 
        enum Role {
            USER
            ADMIN
            GUEST
        }
        "#;

        assert_eq!(
            enum_decl().parse(enum_str).into_result(),
            Ok(Declaration::Enum(EnumDecl {
                name: Token::Ident("Role", Span::new(0, 0)),
                elements: vec![
                    Token::Ident("USER", Span::new(0, 0)),
                    Token::Ident("ADMIN", Span::new(0, 0)),
                    Token::Ident("GUEST", Span::new(0, 0)),
                ]
            }))
        );

        let enum_str_err = r#" 
        enum Role {
            USER,
            ADMIN,
            GUEST,
        }
        "#;

        assert!(enum_decl().parse(enum_str_err).into_result().is_err());

        let empty_enum_str = r#" 
        enum Role {
            
        }
        "#;

        assert!(enum_decl().parse(empty_enum_str).into_result().is_err());
    }

    #[test]
    fn test_field_type() {
        assert_eq!(
            field_type().parse("ShortStr").into_result(),
            Ok(FieldType::new(
                Type::Primitive(PrimitiveType::ShortStr),
                false,
                false,
            ))
        );

        assert_eq!(
            field_type().parse("ShortStr?").into_result(),
            Ok(FieldType::new(
                Type::Primitive(PrimitiveType::ShortStr),
                true,
                false,
            ))
        );

        assert_eq!(
            field_type().parse("ShortStr[]").into_result(),
            Ok(FieldType::new(
                Type::Primitive(PrimitiveType::ShortStr),
                false,
                true,
            ))
        );

        assert_eq!(
            field_type().parse("MyEnum?").into_result(),
            Ok(FieldType::new(
                Type::Unknown(Token::Ident("MyEnum", Span::new(0, 0))),
                true,
                false,
            ))
        );

        assert!(field_type().parse("MyEnum?[]").into_result().is_err());
    }

    #[test]
    fn test_attribute() {
        assert_eq!(
            attribute().parse("@unique").into_result(),
            Ok(Attribute {
                name: Token::Ident("unique", Span::new(0, 0)),
                arg: None
            })
        );

        assert_eq!(
            attribute().parse("@default(now())").into_result(),
            Ok(Attribute {
                name: Token::Ident("default", Span::new(0, 0)),
                arg: Some(AttribArg {
                    name: Token::Ident("now", Span::new(0, 0)),
                    is_function: true
                })
            })
        );

        assert_eq!(
            attribute().parse("@default(USER)").into_result(),
            Ok(Attribute {
                name: Token::Ident("default", Span::new(0, 0)),
                arg: Some(AttribArg {
                    name: Token::Ident("USER", Span::new(0, 0)),
                    is_function: false
                })
            })
        );

        assert!(attribute().parse("unique").into_result().is_err());
        assert!(attribute().parse("@default()").into_result().is_err());
        assert!(attribute().parse("@default(@some)").into_result().is_err());
        assert!(attribute()
            .parse("@default(now(), now())")
            .into_result()
            .is_err());
    }

    #[test]
    fn test_field_dec() {
        assert_eq!(
            field_decl()
                .parse("   id          ShortStr?       @unique_id @default(auto_generate())\n")
                .into_result(),
            Ok(FieldDecl {
                name: Token::Ident("id", Span::new(0, 0)),
                field_type: FieldType::new(Type::Primitive(PrimitiveType::ShortStr), true, false),
                attributes: vec![
                    Attribute {
                        name: Token::Ident("unique_id", Span::new(0, 0)),
                        arg: None
                    },
                    Attribute {
                        name: Token::Ident("default", Span::new(0, 0)),
                        arg: Some(AttribArg {
                            name: Token::Ident("auto_generate", Span::new(0, 0)),
                            is_function: true
                        })
                    }
                ]
            })
        );

        assert_eq!(
            field_decl()
                .parse("   id          ShortStr?       \n")
                .into_result(),
            Ok(FieldDecl {
                name: Token::Ident("id", Span::new(0, 0)),
                field_type: FieldType::new(Type::Primitive(PrimitiveType::ShortStr), true, false),
                attributes: vec![]
            })
        );

        assert!(field_decl().parse("   id    ").into_result().is_err());
    }

    #[test]
    fn test_model_dec() {
        let model_str = r#" 
        model User {
            email       ShortStr      @unique
            name        ShortStr?
            nick_names  ShortStr[]
            role        Role          @default(USER)
        }
        "#;
        assert_eq!(
            model_decl().parse(model_str).into_result(),
            Ok(Declaration::Model(ModelDecl {
                name: Token::Ident("User", Span::new(0, 0)),
                fields: vec![
                    FieldDecl {
                        name: Token::Ident("email", Span::new(0, 0)),
                        field_type: FieldType::new(
                            Type::Primitive(PrimitiveType::ShortStr),
                            false,
                            false
                        ),
                        attributes: vec![Attribute {
                            name: Token::Ident("unique", Span::new(0, 0)),
                            arg: None
                        }]
                    },
                    FieldDecl {
                        name: Token::Ident("name", Span::new(0, 0)),
                        field_type: FieldType::new(
                            Type::Primitive(PrimitiveType::ShortStr),
                            true,
                            false
                        ),
                        attributes: vec![]
                    },
                    FieldDecl {
                        name: Token::Ident("nick_names", Span::new(0, 0)),
                        field_type: FieldType::new(
                            Type::Primitive(PrimitiveType::ShortStr),
                            false,
                            true
                        ),
                        attributes: vec![]
                    },
                    FieldDecl {
                        name: Token::Ident("role", Span::new(0, 0)),
                        field_type: FieldType::new(
                            Type::Unknown(Token::Ident("Role", Span::new(0, 0))),
                            false,
                            false
                        ),
                        attributes: vec![Attribute {
                            name: Token::Ident("default", Span::new(0, 0)),
                            arg: Some(AttribArg {
                                name: Token::Ident("USER", Span::new(0, 0)),
                                is_function: false,
                            })
                        }]
                    }
                ]
            }))
        );

        let empty_model_str = r#"
            model EmptyModel {

            }
        "#;
        assert_eq!(
            model_decl().parse(empty_model_str).into_result(),
            Ok(Declaration::Model(ModelDecl {
                name: Token::Ident("EmptyModel", Span::new(0, 0)),
                fields: vec![]
            }))
        );

        let err_model_str = r#" 
        model User {
            email       ShortStr      @unique
            name        ShortStrnick_names  ShortStr[]
            role        Role          @default(USER)
        }
        "#;

        assert!(model_decl().parse(err_model_str).into_result().is_err());
    }

    #[test]
    fn test_parse() {
        let sdml_str = r#" 
        model User {
            email       ShortStr      @unique
            name        ShortStr?
            nick_names  ShortStr[]
            role        Role          @default(USER)
            mentor      User
        }

        model EmptyModel {

        }
        enum Role {
            USER
            ADMIN
            GUEST
        }
        enum Role1 {
            USER1
            ADMIN1
            GUEST1
        }
        config db {
            provider = "foundationDB"
            port = 1233
            time_out_in_secs = 12.10
        }
        config db1 {
            provider = "foundationDB"
            port = 1233
            time_out_in_secs = 12.10
        }
        "#;

        let declarations = vec![
            Declaration::Model(ModelDecl {
                name: Token::Ident("User", Span::new(0, 0)),
                fields: vec![
                    FieldDecl {
                        name: Token::Ident("email", Span::new(0, 0)),
                        field_type: FieldType::new(
                            Type::Primitive(PrimitiveType::ShortStr),
                            false,
                            false,
                        ),
                        attributes: vec![Attribute {
                            name: Token::Ident("unique", Span::new(0, 0)),
                            arg: None,
                        }],
                    },
                    FieldDecl {
                        name: Token::Ident("name", Span::new(0, 0)),
                        field_type: FieldType::new(
                            Type::Primitive(PrimitiveType::ShortStr),
                            true,
                            false,
                        ),
                        attributes: vec![],
                    },
                    FieldDecl {
                        name: Token::Ident("nick_names", Span::new(0, 0)),
                        field_type: FieldType::new(
                            Type::Primitive(PrimitiveType::ShortStr),
                            false,
                            true,
                        ),
                        attributes: vec![],
                    },
                    FieldDecl {
                        name: Token::Ident("role", Span::new(0, 0)),
                        field_type: FieldType::new(
                            Type::Enum(Token::Ident("Role", Span::new(0, 0))),
                            false,
                            false,
                        ),
                        attributes: vec![Attribute {
                            name: Token::Ident("default", Span::new(0, 0)),
                            arg: Some(AttribArg {
                                name: Token::Ident("USER", Span::new(0, 0)),
                                is_function: false,
                            }),
                        }],
                    },
                    FieldDecl {
                        name: Token::Ident("mentor", Span::new(0, 0)),
                        field_type: FieldType::new(
                            Type::Relation(Token::Ident("User", Span::new(0, 0))),
                            false,
                            false,
                        ),
                        attributes: vec![],
                    },
                ],
            }),
            Declaration::Model(ModelDecl {
                name: Token::Ident("EmptyModel", Span::new(0, 0)),
                fields: vec![],
            }),
            Declaration::Enum(EnumDecl {
                name: Token::Ident("Role", Span::new(0, 0)),
                elements: vec![
                    Token::Ident("USER", Span::new(0, 0)),
                    Token::Ident("ADMIN", Span::new(0, 0)),
                    Token::Ident("GUEST", Span::new(0, 0)),
                ],
            }),
            Declaration::Enum(EnumDecl {
                name: Token::Ident("Role1", Span::new(0, 0)),
                elements: vec![
                    Token::Ident("USER1", Span::new(0, 0)),
                    Token::Ident("ADMIN1", Span::new(0, 0)),
                    Token::Ident("GUEST1", Span::new(0, 0)),
                ],
            }),
            Declaration::Config(ConfigDecl {
                name: Token::Ident("db", Span::new(0, 0)),
                config_pairs: vec![
                    ConfigPair {
                        name: Token::Ident("provider", Span::new(0, 0)),
                        value: ConfigValue::Str("\"foundationDB\"", Span::new(0, 0)),
                    },
                    ConfigPair {
                        name: Token::Ident("port", Span::new(0, 0)),
                        value: ConfigValue::Int(1233, Span::new(0, 0)),
                    },
                    ConfigPair {
                        name: Token::Ident("time_out_in_secs", Span::new(0, 0)),
                        value: ConfigValue::Float(12.10, Span::new(0, 0)),
                    },
                ],
            }),
            Declaration::Config(ConfigDecl {
                name: Token::Ident("db1", Span::new(0, 0)),
                config_pairs: vec![
                    ConfigPair {
                        name: Token::Ident("provider", Span::new(0, 0)),
                        value: ConfigValue::Str("\"foundationDB\"", Span::new(0, 0)),
                    },
                    ConfigPair {
                        name: Token::Ident("port", Span::new(0, 0)),
                        value: ConfigValue::Int(1233, Span::new(0, 0)),
                    },
                    ConfigPair {
                        name: Token::Ident("time_out_in_secs", Span::new(0, 0)),
                        value: ConfigValue::Float(12.10, Span::new(0, 0)),
                    },
                ],
            }),
        ];

        let mut ast: DataModel = DataModel {
            configs: HashMap::new(),
            enums: HashMap::new(),
            models: HashMap::new(),
        };
        declarations.into_iter().for_each(|decl| match decl {
            Declaration::Config(c) => {
                ast.configs.insert(c.name.ident_name().unwrap(), c);
            }
            Declaration::Enum(e) => {
                ast.enums.insert(e.name.ident_name().unwrap(), e);
            }
            Declaration::Model(m) => {
                ast.models.insert(m.name.ident_name().unwrap(), m);
            }
        });
        assert_eq!(new().parse(sdml_str).into_result(), Ok(ast));
    }

    #[test]
    fn test_parse_1() {
        let sdml_str = r#"
            config db {
                provider = "foundationDB"
            }

            model User {
                email       ShortStr      @unique
                name        ShortStr?     
                nick_names  ShortStr[]
                role        Role          @default(USER)
                profile     Profile?
                posts       Post[]
            }

            model Profile {
                bio        LongStr?
                user       User
            }

            model Post {
                createdAt   DateTime    @default(now())
                updatedAt   DateTime
                title       ShortStr
                published   Boolean
                author      User
                category    Category[]
            }

            model Category {
                name        ShortStr
                posts       Post[]
            }

            enum Role {
                USER
                ADMIN
            }
        "#;

        match new().parse(sdml_str).into_result() {
            Ok(ast) => println!("{ast:#?}"),
            Err(errs) => {
                println!("{errs:#?}");
                assert!(false);
            }
        }
    }
}
