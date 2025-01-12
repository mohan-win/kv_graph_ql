use std::ops::Div;

use crate::types::{
  AttribArg, Attribute, ConfigDecl, ConfigPair, DataModel, Declaration, EnumDecl,
  FieldDecl, FieldType, FieldTypeMod, ModelDecl, NamedArg, PrimitiveType, Str, Token,
  Type,
};
use chumsky::text::{self, ascii};
use chumsky::{extra::Err, prelude::*};

pub(crate) mod semantic_analysis;
use semantic_analysis::err::Error;

pub fn parse(src: &str) -> Result<DataModel, Vec<Error>> {
  let decls = delcarations()
    .parse(src)
    .into_result()
    .map_err(|e| e.into_iter().map(Error::from).collect::<Vec<_>>())?;
  semantic_analysis::semantic_update(decls)
}

#[inline(always)]
pub(crate) fn delcarations<'src>(
) -> impl Parser<'src, &'src str, Vec<Declaration>, Err<Rich<'src, char>>> {
  config_decl()
    .or(enum_decl())
    .or(model_decl())
    .repeated()
    .collect::<Vec<Declaration>>()
}

#[inline(always)]
fn config_decl<'src>() -> impl Parser<'src, &'src str, Declaration, Err<Rich<'src, char>>>
{
  text::keyword("config")
    .padded()
    .then(
      ascii::ident()
        .map_with(|tok, e| Token::Ident(Str::new(tok), e.span()))
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
fn config_pair<'src>() -> impl Parser<'src, &'src str, ConfigPair, Err<Rich<'src, char>>>
{
  ascii::ident()
    .map_with(|ident, e| Token::Ident(Str::new(ident), e.span()))
    .padded()
    .then(just('=').padded())
    .then(bool().or(number()).or(string()).padded())
    .map(|((key, _), value): ((Token, char), Token)| ConfigPair {
      name: key,
      value: value.try_into().unwrap(),
    })
}

#[inline(always)]
fn string<'src>() -> impl Parser<'src, &'src str, Token, Err<Rich<'src, char>>> {
  just('"')
    .then(any().filter(|c: &char| *c != '"').repeated())
    .then(just('"'))
    .to_slice()
    .map_with(|str, e| Token::String(Str::new(str), e.span()))
}

#[inline(always)]
fn number<'src>() -> impl Parser<'src, &'src str, Token, Err<Rich<'src, char>>> {
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
fn bool<'src>() -> impl Parser<'src, &'src str, Token, Err<Rich<'src, char>>> {
  text::keyword("true")
    .or(text::keyword("false"))
    .to_slice()
    .map_with(|b: &str, e| Token::Bool(b.parse().unwrap(), e.span()))
}

#[inline(always)]
fn enum_decl<'src>() -> impl Parser<'src, &'src str, Declaration, Err<Rich<'src, char>>> {
  let identifier =
    ascii::ident().map_with(|tok, e| Token::Ident(Str::new(tok), e.span()));
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
fn model_decl<'src>() -> impl Parser<'src, &'src str, Declaration, Err<Rich<'src, char>>>
{
  text::keyword("model")
    .padded()
    .then(ascii::ident().padded())
    .then(just('{'))
    .then(field_decl().repeated().collect::<Vec<FieldDecl>>())
    .then(just('}').padded())
    .map_with(
      |((((_model, name), _open_brace), fields), _close_brace), e| {
        Declaration::Model(ModelDecl {
          name: Token::Ident(Str::new(name), e.span()),
          fields,
        })
      },
    )
}

#[inline(always)]
fn field_decl<'src>() -> impl Parser<'src, &'src str, FieldDecl, Err<Rich<'src, char>>> {
  ascii::ident()
    .padded()
    .map_with(|tok, e| Token::Ident(Str::new(tok), e.span()))
    .then(field_type().padded())
    .then(attribute().padded().repeated().collect::<Vec<Attribute>>())
    .map(|((name, field_type), attributes)| FieldDecl {
      name,
      field_type,
      attributes,
    })
}

#[inline(always)]
fn field_type<'src>() -> impl Parser<'src, &'src str, FieldType, Err<Rich<'src, char>>> {
  let primitive_type = text::keyword("ShortStr")
    .or(text::keyword("LongStr"))
    .or(text::keyword("DateTime"))
    .or(text::keyword("Boolean"))
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
        "Boolean" => Some(PrimitiveType::Boolean),
        "Int32" => Some(PrimitiveType::Int32),
        "Int64" => Some(PrimitiveType::Int64),
        "Float64" => Some(PrimitiveType::Float64),
        _ => None,
      }
      .map_or(
        Type::Unknown(Token::Ident(Str::new(r#type), e.span())),
        |primitive_type| Type::Primitive {
          r#type: primitive_type,
          token: Token::Ident(Str::new(r#type), e.span()),
        },
      );
      let field_type_mod =
        optional_or_array.map_or(FieldTypeMod::NonOptional, |v| match v {
          "?" => FieldTypeMod::Optional,
          "[]" => FieldTypeMod::Array,
          _ => panic!("Unexpected type modifier!"),
        });
      FieldType::new(r#type, field_type_mod)
    })
}

#[inline(always)]
fn named_arg<'src>() -> impl Parser<'src, &'src str, NamedArg, Err<Rich<'src, char>>> {
  let identifier =
    ascii::ident().map_with(|tok, e| Token::Ident(Str::new(tok), e.span()));
  identifier
    .padded()
    .then(just(":"))
    .then(identifier.or(string()).padded())
    .map_with(|((arg_name, _colon), arg_value), _e| NamedArg {
      arg_name,
      arg_value,
    })
}

#[inline(always)]
fn attribute<'src>() -> impl Parser<'src, &'src str, Attribute, Err<Rich<'src, char>>> {
  let arg_list = named_arg()
    .then(
      just(",")
        .ignore_then(named_arg())
        .repeated()
        .collect::<Vec<NamedArg>>(),
    )
    .map(|(first_arg, mut more_args): (NamedArg, Vec<NamedArg>)| {
      more_args.insert(0, first_arg);
      AttribArg::Args(more_args)
    });
  let function = ascii::ident()
    .then(just("()"))
    .map_with(|(func_name, _parans), e| {
      AttribArg::Function(Token::Ident(Str::new(func_name), e.span()))
    });

  let identifier = ascii::ident()
    .map_with(|tok, e| AttribArg::Ident(Token::Ident(Str::new(tok), e.span())));

  let attribute_arg = arg_list.or(function).or(identifier);
  just('@')
    .then(ascii::ident())
    .then(just('(').then(attribute_arg).then(just(')')).or_not())
    .map_with(|((_at, name), arg), e| {
      let arg = arg.map(|((_open_paran, attrib_arg), _close_paran)| attrib_arg);
      Attribute {
        name: Token::Ident(Str::new(name), e.span()),
        arg,
      }
    })
}

#[cfg(test)]
mod tests {
  use std::{collections::HashMap, vec};

  use super::*;
  use crate::types::{ConfigValue, RelationEdge, Span};

  #[test]
  fn test_string() {
    assert_eq!(
      string().parse("\"Valid String\"").into_result(),
      Ok(Token::String(Str::new("\"Valid String\""), Span::new(0, 0)))
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
        name: Token::Ident(Str::new("provider"), Span::new(0, 0)),
        value: ConfigValue::String(Str::new("\"foundationDB\""), Span::new(0, 0))
      })
    );

    assert_eq!(
      config_pair()
        .parse("\n   provider   =   \"foundationDB\"  \n")
        .into_result(),
      Ok(ConfigPair {
        name: Token::Ident(Str::new("provider"), Span::new(0, 0)),
        value: ConfigValue::String(Str::new("\"foundationDB\""), Span::new(0, 0))
      })
    );

    assert_eq!(
      config_pair()
        .parse("\n   is_local   =   true  \n")
        .into_result(),
      Ok(ConfigPair {
        name: Token::Ident(Str::new("is_local"), Span::new(0, 0)),
        value: ConfigValue::Bool(true, Span::new(0, 0))
      })
    );

    assert_eq!(
      config_pair()
        .parse("\n   port   =   1233  \n")
        .into_result(),
      Ok(ConfigPair {
        name: Token::Ident(Str::new("port"), Span::new(0, 0)),
        value: ConfigValue::Int(1233, Span::new(0, 0))
      })
    );

    assert_eq!(
      config_pair()
        .parse("\n   time_out_in_secs   =   10.25  \n")
        .into_result(),
      Ok(ConfigPair {
        name: Token::Ident(Str::new("time_out_in_secs"), Span::new(0, 0)),
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
        name: Token::Ident(Str::new("db"), Span::new(0, 0)),
        config_pairs: vec![
          ConfigPair {
            name: Token::Ident(Str::new("provider"), Span::new(0, 0)),
            value: ConfigValue::String(Str::new("\"foundationDB\""), Span::new(0, 0))
          },
          ConfigPair {
            name: Token::Ident(Str::new("port"), Span::new(0, 0)),
            value: ConfigValue::Int(1233, Span::new(0, 0))
          },
          ConfigPair {
            name: Token::Ident(Str::new("time_out_in_secs"), Span::new(0, 0)),
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
        name: Token::Ident(Str::new("Role"), Span::new(0, 0)),
        elements: vec![
          Token::Ident(Str::new("USER"), Span::new(0, 0)),
          Token::Ident(Str::new("ADMIN"), Span::new(0, 0)),
          Token::Ident(Str::new("GUEST"), Span::new(0, 0)),
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
      field_type().parse("Boolean").into_result(),
      Ok(FieldType::new(
        Type::Primitive {
          r#type: PrimitiveType::Boolean,
          token: Token::Ident(Str::new("Boolean"), Span::new(0, 0))
        },
        FieldTypeMod::NonOptional
      ))
    );

    assert_eq!(
      field_type().parse("ShortStr").into_result(),
      Ok(FieldType::new(
        Type::Primitive {
          r#type: PrimitiveType::ShortStr,
          token: Token::Ident(Str::new("ShortStr"), Span::new(0, 0))
        },
        FieldTypeMod::NonOptional
      ))
    );

    assert_eq!(
      field_type().parse("ShortStr?").into_result(),
      Ok(FieldType::new(
        Type::Primitive {
          r#type: PrimitiveType::ShortStr,
          token: Token::Ident(Str::new("ShortStr"), Span::new(0, 0))
        },
        FieldTypeMod::Optional
      ))
    );

    assert_eq!(
      field_type().parse("ShortStr[]").into_result(),
      Ok(FieldType::new(
        Type::Primitive {
          r#type: PrimitiveType::ShortStr,
          token: Token::Ident(Str::new("ShortStr"), Span::new(0, 0)),
        },
        FieldTypeMod::Array
      ))
    );

    assert_eq!(
      field_type().parse("MyEnum?").into_result(),
      Ok(FieldType::new(
        Type::Unknown(Token::Ident(Str::new("MyEnum"), Span::new(0, 0))),
        FieldTypeMod::Optional
      ))
    );

    assert!(field_type().parse("MyEnum?[]").into_result().is_err());
  }

  #[test]
  fn test_attribute() {
    assert_eq!(
      attribute().parse("@unique").into_result(),
      Ok(Attribute {
        name: Token::Ident(Str::new("unique"), Span::new(0, 0)),
        arg: None
      })
    );

    assert_eq!(
      attribute().parse("@default(now())").into_result(),
      Ok(Attribute {
        name: Token::Ident(Str::new("default"), Span::new(0, 0)),
        arg: Some(AttribArg::Function(Token::Ident(
          Str::new("now"),
          Span::new(0, 0)
        )))
      })
    );

    assert_eq!(
      attribute().parse("@default(USER)").into_result(),
      Ok(Attribute {
        name: Token::Ident(Str::new("default"), Span::new(0, 0)),
        arg: Some(AttribArg::Ident(Token::Ident(
          Str::new("USER"),
          Span::new(0, 0)
        )))
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
  fn test_attribute_with_attrib_args() {
    assert_eq!(
      attribute()
        .parse(r#"@relation(name: "MyRelation")"#)
        .into_result(),
      Ok(Attribute {
        name: Token::Ident(Str::new("relation"), Span::new(0, 0)),
        arg: Some(AttribArg::Args(vec![NamedArg {
          arg_name: Token::Ident(Str::new("name"), Span::new(0, 0)),
          arg_value: Token::String(Str::new("\"MyRelation\""), Span::new(0, 0))
        }]))
      })
    );

    assert_eq!(
      attribute()
        .parse(r#"@relation(name: "MyRelation", field: field1, references: id1)"#)
        .into_result(),
      Ok(Attribute {
        name: Token::Ident(Str::new("relation"), Span::new(0, 0)),
        arg: Some(AttribArg::Args(vec![
          NamedArg {
            arg_name: Token::Ident(Str::new("name"), Span::new(0, 0)),
            arg_value: Token::String(Str::new("\"MyRelation\""), Span::new(0, 0))
          },
          NamedArg {
            arg_name: Token::Ident(Str::new("field"), Span::new(0, 0)),
            arg_value: Token::Ident(Str::new("field1"), Span::new(0, 0))
          },
          NamedArg {
            arg_name: Token::Ident(Str::new("references"), Span::new(0, 0)),
            arg_value: Token::Ident(Str::new("id1"), Span::new(0, 0))
          }
        ]))
      })
    );

    assert!(attribute()
      .parse(r#"@attribute_with_out_arg_name("my_relation")"#)
      .into_result()
      .is_err());
    assert!(attribute()
            .parse(r#"@attribute_with_out_arg_name("my_relation", field: scalar_field1, references: references_id1)"#)
            .into_result()
            .is_err());
    assert!(attribute()
      .parse(
        r#"@invalid_relation(now(), name:"my_relation", field: field1, references: id1)"#
      )
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
        name: Token::Ident(Str::new("id"), Span::new(0, 0)),
        field_type: FieldType::new(
          Type::Primitive {
            r#type: PrimitiveType::ShortStr,
            token: Token::Ident(Str::new("ShortStr"), Span::new(0, 0))
          },
          FieldTypeMod::Optional
        ),
        attributes: vec![
          Attribute {
            name: Token::Ident(Str::new("unique_id"), Span::new(0, 0)),
            arg: None
          },
          Attribute {
            name: Token::Ident(Str::new("default"), Span::new(0, 0)),
            arg: Some(AttribArg::Function(Token::Ident(
              Str::new("auto_generate"),
              Span::new(0, 0)
            )))
          }
        ]
      })
    );

    assert_eq!(
      field_decl()
        .parse("   id          ShortStr?       \n")
        .into_result(),
      Ok(FieldDecl {
        name: Token::Ident(Str::new("id"), Span::new(0, 0)),
        field_type: FieldType::new(
          Type::Primitive {
            r#type: PrimitiveType::ShortStr,
            token: Token::Ident(Str::new("ShortStr"), Span::new(0, 0))
          },
          FieldTypeMod::Optional
        ),
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
            nickNames  ShortStr[]
            role        Role          @default(USER)
        }
        "#;
    assert_eq!(
      model_decl().parse(model_str).into_result(),
      Ok(Declaration::Model(ModelDecl {
        name: Token::Ident(Str::new("User"), Span::new(0, 0)),
        fields: vec![
          FieldDecl {
            name: Token::Ident(Str::new("email"), Span::new(0, 0)),
            field_type: FieldType::new(
              Type::Primitive {
                r#type: (PrimitiveType::ShortStr),
                token: Token::Ident(Str::new("ShortStr"), Span::new(0, 0))
              },
              FieldTypeMod::NonOptional
            ),
            attributes: vec![Attribute {
              name: Token::Ident(Str::new("unique"), Span::new(0, 0)),
              arg: None
            }]
          },
          FieldDecl {
            name: Token::Ident(Str::new("name"), Span::new(0, 0)),
            field_type: FieldType::new(
              Type::Primitive {
                r#type: (PrimitiveType::ShortStr),
                token: Token::Ident(Str::new("ShortStr"), Span::new(0, 0))
              },
              FieldTypeMod::Optional
            ),
            attributes: vec![]
          },
          FieldDecl {
            name: Token::Ident(Str::new("nickNames"), Span::new(0, 0)),
            field_type: FieldType::new(
              Type::Primitive {
                r#type: (PrimitiveType::ShortStr),
                token: Token::Ident(Str::new("ShortStr"), Span::new(0, 0))
              },
              FieldTypeMod::Array
            ),
            attributes: vec![]
          },
          FieldDecl {
            name: Token::Ident(Str::new("role"), Span::new(0, 0)),
            field_type: FieldType::new(
              Type::Unknown(Token::Ident(Str::new("Role"), Span::new(0, 0))),
              FieldTypeMod::NonOptional
            ),
            attributes: vec![Attribute {
              name: Token::Ident(Str::new("default"), Span::new(0, 0)),
              arg: Some(AttribArg::Ident(Token::Ident(
                Str::new("USER"),
                Span::new(0, 0)
              )))
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
        name: Token::Ident(Str::new("EmptyModel"), Span::new(0, 0)),
        fields: vec![]
      }))
    );

    let err_model_str = r#"
        model User {
            email       ShortStr      @unique
            name        ShortStrnickNames  ShortStr[]
            role        Role          @default(USER)
        }
        "#;

    assert!(model_decl().parse(err_model_str).into_result().is_err());
  }

  #[test]
  fn test_parse() {
    let sdml_str = r#"
        model User {
            id           ShortStr      @id @default(auto())
            email        ShortStr      @unique
            name         ShortStr?
            nickNames    ShortStr[]
            role         Role          @default(USER)
            mentor       User?         @relation(name: "UserMentor", field: mentorEmail, references: email)
            mentorEmail  ShortStr?
            mentees      User[]        @relation(name: "UserMentor")
        }

        model EmptyModel {
            id  ShortStr   @id
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
        "#.to_string();
    let mut configs = HashMap::new();
    let mut enums = HashMap::new();
    let mut models = HashMap::new();
    let mut relations = HashMap::new();
    models.insert(
      "User".to_string(),
      ModelDecl {
        name: Token::Ident(Str::new("User"), Span::new(0, 0)),
        fields: vec![
          FieldDecl {
            name: Token::Ident(Str::new("id"), Span::new(0, 0)),
            field_type: FieldType::new(
              Type::Primitive {
                r#type: PrimitiveType::ShortStr,
                token: Token::Ident(Str::new("ShortStr"), Span::new(0, 0)),
              },
              FieldTypeMod::NonOptional,
            ),
            attributes: vec![
              Attribute {
                name: Token::Ident(Str::new("id"), Span::new(0, 0)),
                arg: None,
              },
              Attribute {
                name: Token::Ident(Str::new("default"), Span::new(0, 0)),
                arg: Some(AttribArg::Function(Token::Ident(
                  Str::new("auto"),
                  Span::new(0, 0),
                ))),
              },
            ],
          },
          FieldDecl {
            name: Token::Ident(Str::new("email"), Span::new(0, 0)),
            field_type: FieldType::new(
              Type::Primitive {
                r#type: (PrimitiveType::ShortStr),
                token: Token::Ident(Str::new("ShortStr"), Span::new(0, 0)),
              },
              FieldTypeMod::NonOptional,
            ),
            attributes: vec![Attribute {
              name: Token::Ident(Str::new("unique"), Span::new(0, 0)),
              arg: None,
            }],
          },
          FieldDecl {
            name: Token::Ident(Str::new("name"), Span::new(0, 0)),
            field_type: FieldType::new(
              Type::Primitive {
                r#type: (PrimitiveType::ShortStr),
                token: Token::Ident(Str::new("ShortStr"), Span::new(0, 0)),
              },
              FieldTypeMod::Optional,
            ),
            attributes: vec![],
          },
          FieldDecl {
            name: Token::Ident(Str::new("nickNames"), Span::new(0, 0)),
            field_type: FieldType::new(
              Type::Primitive {
                r#type: (PrimitiveType::ShortStr),
                token: Token::Ident(Str::new("ShortStr"), Span::new(0, 0)),
              },
              FieldTypeMod::Array,
            ),
            attributes: vec![],
          },
          FieldDecl {
            name: Token::Ident(Str::new("role"), Span::new(0, 0)),
            field_type: FieldType::new(
              Type::Enum {
                enum_ty_name: Token::Ident(Str::new("Role"), Span::new(0, 0)),
              },
              FieldTypeMod::NonOptional,
            ),
            attributes: vec![Attribute {
              name: Token::Ident(Str::new("default"), Span::new(0, 0)),
              arg: Some(AttribArg::Ident(Token::Ident(
                Str::new("USER"),
                Span::new(0, 0),
              ))),
            }],
          },
          FieldDecl {
            name: Token::Ident(Str::new("mentor"), Span::new(0, 0)),
            field_type: FieldType::new(
              Type::Relation(RelationEdge::ManySideRelation {
                relation_name: Token::String(
                  Str::new(r#""UserMentor""#),
                  Span::new(0, 0),
                ),
                scalar_field_name: Token::Ident(Str::new("mentorEmail"), Span::new(0, 0)),
                referenced_model_name: Token::Ident(Str::new("User"), Span::new(0, 0)),
                referenced_field_name: Token::Ident(Str::new("email"), Span::new(0, 0)),
              }),
              FieldTypeMod::Optional,
            ),
            attributes: vec![Attribute {
              name: Token::Ident(Str::new("relation"), Span::new(0, 0)),
              arg: Some(AttribArg::Args(vec![
                NamedArg {
                  arg_name: Token::Ident(Str::new("name"), Span::new(0, 0)),
                  arg_value: Token::String(Str::new("\"UserMentor\""), Span::new(0, 0)),
                },
                NamedArg {
                  arg_name: Token::Ident(Str::new("field"), Span::new(0, 0)),
                  arg_value: Token::Ident(Str::new("mentorEmail"), Span::new(0, 0)),
                },
                NamedArg {
                  arg_name: Token::Ident(Str::new("references"), Span::new(0, 0)),
                  arg_value: Token::Ident(Str::new("email"), Span::new(0, 0)),
                },
              ])),
            }],
          },
          FieldDecl {
            name: Token::Ident(Str::new("mentorEmail"), Span::new(0, 0)),
            field_type: FieldType::new(
              Type::Primitive {
                r#type: PrimitiveType::ShortStr,
                token: Token::Ident(Str::new("ShortStr"), Span::new(0, 0)),
              },
              FieldTypeMod::Optional,
            ),
            attributes: vec![],
          },
          FieldDecl {
            name: Token::Ident(Str::new("mentees"), Span::new(0, 0)),
            field_type: FieldType::new(
              Type::Relation(RelationEdge::OneSideRelation {
                relation_name: Token::String(Str::new("\"UserMentor\""), Span::new(0, 0)),
                referenced_model_name: Token::Ident(Str::new("User"), Span::new(0, 0)),
              }),
              FieldTypeMod::Array,
            ),
            attributes: vec![Attribute {
              name: Token::Ident(Str::new("relation"), Span::new(0, 0)),
              arg: Some(AttribArg::Args(vec![NamedArg {
                arg_name: Token::Ident(Str::new("name"), Span::new(0, 0)),
                arg_value: Token::String(Str::new("\"UserMentor\""), Span::new(0, 0)),
              }])),
            }],
          },
        ],
      },
    );
    models.insert(
      "EmptyModel".to_string(),
      ModelDecl {
        name: Token::Ident(Str::new("EmptyModel"), Span::new(0, 0)),
        fields: vec![FieldDecl {
          name: Token::Ident(Str::new("id"), Span::new(0, 0)),
          field_type: FieldType::new(
            Type::Primitive {
              r#type: PrimitiveType::ShortStr,
              token: Token::Ident(Str::new("ShortStr"), Span::new(0, 0)),
            },
            FieldTypeMod::NonOptional,
          ),
          attributes: vec![Attribute {
            name: Token::Ident(Str::new("id"), Span::new(0, 0)),
            arg: None,
          }],
        }],
      },
    );
    relations.insert(
      "UserMentor".to_string(),
      (
        RelationEdge::OneSideRelation {
          relation_name: Token::String(Str::new(r#""UserMentor""#), Span::new(0, 0)),
          referenced_model_name: Token::Ident(Str::new("User"), Span::new(0, 0)),
        },
        Some(RelationEdge::ManySideRelation {
          relation_name: Token::String(Str::new(r#""UserMentor""#), Span::new(0, 0)),
          scalar_field_name: Token::Ident(Str::new("mentorEmail"), Span::new(0, 0)),
          referenced_model_name: Token::Ident(Str::new("User"), Span::new(0, 0)),
          referenced_field_name: Token::Ident(Str::new("email"), Span::new(0, 0)),
        }),
      ),
    );
    enums.insert(
      "Role".to_string(),
      EnumDecl {
        name: Token::Ident(Str::new("Role"), Span::new(0, 0)),
        elements: vec![
          Token::Ident(Str::new("USER"), Span::new(0, 0)),
          Token::Ident(Str::new("ADMIN"), Span::new(0, 0)),
          Token::Ident(Str::new("GUEST"), Span::new(0, 0)),
        ],
      },
    );
    enums.insert(
      "Role1".to_string(),
      EnumDecl {
        name: Token::Ident(Str::new("Role1"), Span::new(0, 0)),
        elements: vec![
          Token::Ident(Str::new("USER1"), Span::new(0, 0)),
          Token::Ident(Str::new("ADMIN1"), Span::new(0, 0)),
          Token::Ident(Str::new("GUEST1"), Span::new(0, 0)),
        ],
      },
    );
    configs.insert(
      "db".to_string(),
      ConfigDecl {
        name: Token::Ident(Str::new("db"), Span::new(0, 0)),
        config_pairs: vec![
          ConfigPair {
            name: Token::Ident(Str::new("provider"), Span::new(0, 0)),
            value: ConfigValue::String(Str::new("\"foundationDB\""), Span::new(0, 0)),
          },
          ConfigPair {
            name: Token::Ident(Str::new("port"), Span::new(0, 0)),
            value: ConfigValue::Int(1233, Span::new(0, 0)),
          },
          ConfigPair {
            name: Token::Ident(Str::new("time_out_in_secs"), Span::new(0, 0)),
            value: ConfigValue::Float(12.10, Span::new(0, 0)),
          },
        ],
      },
    );
    configs.insert(
      "db1".to_string(),
      ConfigDecl {
        name: Token::Ident(Str::new("db1"), Span::new(0, 0)),
        config_pairs: vec![
          ConfigPair {
            name: Token::Ident(Str::new("provider"), Span::new(0, 0)),
            value: ConfigValue::String(Str::new("\"foundationDB\""), Span::new(0, 0)),
          },
          ConfigPair {
            name: Token::Ident(Str::new("port"), Span::new(0, 0)),
            value: ConfigValue::Int(1233, Span::new(0, 0)),
          },
          ConfigPair {
            name: Token::Ident(Str::new("time_out_in_secs"), Span::new(0, 0)),
            value: ConfigValue::Float(12.10, Span::new(0, 0)),
          },
        ],
      },
    );
    let ast = DataModel::new(configs, enums, models, relations);
    let actual_ast = parse(&sdml_str).unwrap();
    assert_eq!(ast.enums(), actual_ast.enums());
    assert_eq!(ast.relations(), actual_ast.relations());
    assert_eq!(ast.configs(), actual_ast.configs());
    assert_eq!(ast.models(), actual_ast.models());
    assert_eq!(ast, actual_ast);
  }

  #[test]
  fn test_happy_path_parse() {
    let happy_path_model_sdml = std::fs::read_to_string(concat!(
      env!("CARGO_MANIFEST_DIR"),
      "/test_data/happy_path_model.sdml"
    ))
    .unwrap();

    let ast_result = parse(&happy_path_model_sdml);
    eprintln!("{:#?}", ast_result);
    assert!(ast_result.is_ok());
  }
}
