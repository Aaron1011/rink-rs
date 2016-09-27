// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use context::Context;
use number::Number;
use value::Show;
use std::collections::BTreeMap;
use reply::{PropertyReply, SubstanceReply};
use std::ops::{Mul, Div};
use std::iter::once;

#[derive(Debug, Clone)]
pub struct Property {
    pub input: Number,
    pub input_name: String,
    pub output: Number,
    pub output_name: String,
    pub doc: Option<String>,
}

#[derive(Debug, Clone)]
pub struct Substance {
    pub amount: Number,
    pub properties: BTreeMap<String, Property>,
}

impl Substance {
    pub fn to_reply(&self, context: &Context) -> Result<SubstanceReply, String> {
        if self.amount == Number::one() {
            Ok(SubstanceReply {
                properties: self.properties.iter().map(|(k, v)| {
                    PropertyReply {
                        name: k.clone(),
                        input: Some(v.input.to_parts(context)),
                        output: v.output.to_parts(context),
                        doc: v.doc.clone()
                    }
                }).collect(),
            })
        } else {
            let func = |(_k, v): (&String, &Property)| {
                let input = try!((&v.input / &self.amount).ok_or_else(|| format!(
                    "Division by zero: <{}> / <{}>",
                    v.input.show(context),
                    self.amount.show(context)
                )));
                let output = try!((&v.output / &self.amount).ok_or_else(|| format!(
                    "Division by zero: <{}> / <{}>",
                    v.output.show(context),
                    self.amount.show(context)
                )));
                let (name, input, output) = if input.1.len() == 0 {
                    let div = try!(
                        (&v.output / &input).ok_or_else(|| format!(
                            "Division by zero: <{}> / <{}>",
                            v.output.show(context),
                            input.show(context)
                        ))
                    );
                    (v.output_name.clone(), None, div.to_parts(context))
                } else if output.1.len() == 0 {
                    let div = try!(
                        (&v.input / &output).ok_or_else(|| format!(
                            "Division by zero: <{}> / <{}>",
                            v.input.show(context),
                            output.show(context)
                        ))
                    );
                    (v.input_name.clone(), None, div.to_parts(context))
                } else {
                    return Ok(None)
                };
                Ok(Some(PropertyReply {
                    name: name,
                    input: input,
                    output: output,
                    doc: v.doc.clone(),
                }))
            };
            let amount = PropertyReply {
                name: self.amount.to_parts(context).quantity
                    .unwrap_or_else(|| "amount".to_owned()),
                input: None,
                output: self.amount.to_parts(context),
                doc: None,
            };
            Ok(SubstanceReply {
                properties: try!(
                    once(Ok(Some(amount)))
                        .chain(self.properties.iter().map(func))
                        .collect::<Result<Vec<Option<PropertyReply>>, String>>())
                    .into_iter()
                    .filter_map(|x| x)
                    .collect(),
            })
        }
    }
}

impl Show for Substance {
    fn show(&self, context: &Context) -> String {
        match self.to_reply(context) {
            Ok(v) => format!("{}", v),
            Err(e) => e
        }
    }
}

impl<'a, 'b> Mul<&'b Number> for &'a Substance {
    type Output = Result<Substance, String>;

    fn mul(self, other: &'b Number) -> Self::Output {
        Ok(Substance {
            amount: try!((&self.amount * other).ok_or_else(
                || "Multiplication of numbers should not fail".to_owned())),
            properties: self.properties.clone(),
        })
    }
}

impl<'a, 'b> Div<&'b Number> for &'a Substance {
    type Output = Result<Substance, String>;

    fn div(self, other: &'b Number) -> Self::Output {
        Ok(Substance {
            amount: try!((&self.amount / other).ok_or_else(
                || "Division by zero".to_owned())),
            properties: self.properties.clone(),
        })
    }
}
