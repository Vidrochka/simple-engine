use cssparser::{CowRcStr, ParseError, Parser, ParserInput, Token};

use crate::style::{FlexDirection, Margin, MarginBuilder, PaddingBuilder, SizeValue, UIStyle, UIStyleRules, UIStyleRulesBuilder, Unit};


pub struct UICSSSource {
    data: String,
}

impl UICSSSource {
    pub fn new(data: impl Into<String>) -> Self {
        Self {
            data: data.into()
        }
    }

    pub fn build_styles(&self) -> Vec<UIStyle> {
        let mut input = ParserInput::new(&self.data);
        let mut parser = Parser::new(&mut input);

        let mut styles = Vec::new();

        while let Ok(Token::Delim('.')) = parser.next().cloned() {
            let Ok(Token::Ident(class)) = parser.next().cloned() else {
                panic!("Expected class after '.'");
            };

            tracing::info!("Parse class {class}");

            parser.expect_curly_bracket_block().unwrap();

            fn unpack_value_as_unit<'a>(parser: &'a mut Parser<'_, '_>) -> Unit {
                match parser.next().unwrap() {
                    Token::Dimension { value, unit, .. } => {
                        if unit.as_str() == "px" {
                            tracing::info!("Parsed {value}px");
                            Unit::Pixel(*value as u32)
                        } else {
                            panic!("Unsupported unit {unit:?}")
                        }
                    },
                    Token::Percentage { unit_value, .. } => {
                        tracing::info!("Parsed {unit_value}%");
                        Unit::Percent((*unit_value * 100.0) as u16)
                    },
                    Token::Ident(ident) if ident.as_str() == "auto" => {
                        Unit::Percent(100)
                    }
                    token => {
                        panic!("Unsupported token '{token:?}'");
                    }
                }
            }

            fn unpack_size_value_as_unit<'a>(parser: &'a mut Parser<'_, '_>) -> SizeValue {
                match parser.next().unwrap() {
                    Token::Dimension { value, unit, .. } => {
                        if unit.as_str() == "px" {
                            tracing::info!("Parsed {value}px");
                            SizeValue::Unit(Unit::Pixel(*value as u32))
                        } else {
                            panic!("Unsupported unit {unit:?}")
                        }
                    },
                    Token::Percentage { unit_value, .. } => {
                        tracing::info!("Parsed {unit_value}%");
                        SizeValue::Unit(Unit::Percent((*unit_value * 100.0) as u16))
                    },
                    Token::Ident(ident) if ident.as_str() == "auto" => {
                        SizeValue::Auto
                    },
                    Token::Ident(ident) if ident.as_str() == "fit-content" => {
                        SizeValue::FitContent
                    },
                    token => {
                        panic!("Unsupported token '{token:?}'");
                    }
                }
            }

            parser.parse_nested_block(|mut parser|{
                let mut style_builder = UIStyleRulesBuilder::default();

                while let Ok(property) = parser.expect_ident_cloned() {
                    match property.as_str() {
                        "flex-direction" => {
                            parser.expect_colon()?;
    
                            let value = parser.expect_ident()?;
    
                            match value.as_str() {
                                "row" => style_builder.flex_direction(FlexDirection::Row),
                                "col" => style_builder.flex_direction(FlexDirection::Col),
                                v => panic!("Unsupported value '{v}'"),
                            };
    
                            parser.expect_semicolon()?;
                        },
                        "width" => {
                            parser.expect_colon()?;
    
                            let unit = unpack_size_value_as_unit(&mut parser);
    
                            style_builder.width(unit);
    
                            parser.expect_semicolon()?;
                        },
                        "height" => {
                            parser.expect_colon()?;
    
                            let unit = unpack_size_value_as_unit(&mut parser);
    
                            style_builder.height(unit);
    
                            parser.expect_semicolon()?;
                        },
                        "padding" => {
                            parser.expect_colon()?;
    
                            let unit = unpack_value_as_unit(&mut parser);
    
                            style_builder.padding(
                                PaddingBuilder::default()
                                    .left(unit)
                                    .right(unit)
                                    .bottom(unit)
                                    .top(unit)
                                    .build()
                                    .unwrap()
                            );
    
                            parser.expect_semicolon()?;
                        },
                        "margin" => {
                            parser.expect_colon()?;
    
                            let unit = unpack_value_as_unit(&mut parser);
    
                            style_builder.margin(
                                MarginBuilder::default()
                                    .left(unit)
                                    .right(unit)
                                    .bottom(unit)
                                    .top(unit)
                                    .build()
                                    .unwrap()
                            );
    
                            parser.expect_semicolon()?;
                        },
                        "gap" => {
                            parser.expect_colon()?;
    
                            let unit = unpack_value_as_unit(&mut parser);
    
                            style_builder.gap(unit);
    
                            parser.expect_semicolon()?;
                        },
                        "background-color" => {
                            parser.expect_colon()?;

                            let color = match parser.next()? {
                                Token::Ident(color_name) => color_name,
                                Token::Hash(color_hex) => color_hex,
                                Token::IDHash(color_hex) => color_hex,
                                token => panic!("Unsupported color token '{token:?}'")
                            };

                            let color = csscolorparser::parse(color.as_str()).unwrap().to_rgba8();

                            style_builder.background_color([color[0], color[1], color[2]]);

                            parser.expect_semicolon()?;
                        }
                        property => panic!("Unsupported property name '{property:?}'")
                    }
                }

                let rules = style_builder.build().unwrap();
                    
                styles.push(UIStyle {
                    id: class.clone().into(),
                    class: class.to_string(),
                    rules,
                });

                Ok::<(), ParseError<'_, ()>>(())
            }).unwrap();
            
        }

        styles
    }
}