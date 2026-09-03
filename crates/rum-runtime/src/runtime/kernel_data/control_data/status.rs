mod resume_frames;
use resume_frames::*;

use super::*;

impl Session {
    pub(in crate::runtime) fn eval_failure(
        &mut self,
        head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        let [tag, details] = args else {
            return Ok(Expr::call(head.clone(), args.to_vec()));
        };

        let Some(details) = normalize_failure_details_expr(details) else {
            return Ok(Expr::call(head.clone(), args.to_vec()));
        };

        Ok(Expr::call(head.clone(), vec![tag.clone(), details]))
    }

    pub(in crate::runtime) fn eval_failure_q(
        &mut self,
        _head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        let [expr] = args else {
            return Ok(self.invalid_call_with_exact_arg_count("FailureQ", args, 1));
        };

        Ok(system_expr(if is_failure_q_expr(expr) {
            "True"
        } else {
            "False"
        }))
    }

    pub(in crate::runtime) fn eval_success(
        &mut self,
        head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        let [tag, details] = args else {
            return Ok(Expr::call(head.clone(), args.to_vec()));
        };

        let Some(details) = normalize_success_details_expr(details) else {
            return Ok(Expr::call(head.clone(), args.to_vec()));
        };

        Ok(Expr::call(head.clone(), vec![tag.clone(), details]))
    }

    pub(in crate::runtime) fn eval_terminated_evaluation(
        &mut self,
        head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        Ok(Expr::call(head.clone(), args.to_vec()))
    }

    pub(in crate::runtime) fn failure_property_value(
        &mut self,
        failure: &Expr,
        property: &str,
    ) -> Result<Expr, EvalError> {
        self.status_object_projection_value(failure, &Expr::string(property), None)
    }

    pub(in crate::runtime) fn apply_status_object_to_args(
        &mut self,
        head: &Expr,
        args: &[Expr],
    ) -> Result<Option<Expr>, EvalError> {
        let is_failure = is_failure_expr(head);
        let is_success = is_success_expr(head);
        if !is_failure && !is_success {
            return Ok(None);
        }

        if is_failure && !failure_tag_supports_projection(head) {
            return Ok(None);
        }
        if is_success && !success_tag_supports_projection(head) {
            return Ok(None);
        }

        match args {
            [
                list @ Expr::Call {
                    head: list_head,
                    args: properties,
                    ..
                },
            ] if is_system_symbol_name(list_head, heads::structure::LIST) => {
                let mut values = Vec::with_capacity(properties.len());
                for property in properties {
                    values.push(self.status_object_projection_value(head, property, Some(list))?);
                }
                Ok(Some(Expr::call(system_expr("List"), values)))
            }
            [property] => Ok(Some(
                self.status_object_projection_value(head, property, None)?,
            )),
            _ => Ok(None),
        }
    }

    fn status_object_projection_value(
        &mut self,
        status: &Expr,
        property: &Expr,
        missing_property_context: Option<&Expr>,
    ) -> Result<Expr, EvalError> {
        if missing_property_context.is_none() {
            if is_failure_expr(status) {
                if is_system_symbol_name(property, "All") {
                    return Ok(status_object_all_value(status)
                        .unwrap_or_else(|| status_object_missing_expr(status, property.clone())));
                }

                if let Expr::Atom(Atom::String(name)) = property {
                    return match name.as_str() {
                        "Properties" => Ok(failure_properties_value(status).unwrap_or_else(|| {
                            status_object_missing_expr(status, property.clone())
                        })),
                        "HeldMessageTemplate" => {
                            Ok(failure_virtual_held_message_template_value(status)
                                .or_else(|| status_object_property_value(status, property))
                                .unwrap_or_else(|| {
                                    status_object_missing_expr(status, property.clone())
                                }))
                        }
                        "Message" => Ok(self
                            .failure_virtual_message_value(status)?
                            .unwrap_or_else(|| {
                                status_object_missing_expr(status, property.clone())
                            })),
                        "MessageName" => Ok(failure_virtual_message_name_value(status)
                            .or_else(|| status_object_property_value(status, property))
                            .unwrap_or_else(|| {
                                status_object_missing_expr(status, property.clone())
                            })),
                        "StyledMessage" => Ok(self
                            .failure_virtual_styled_message_value(status)?
                            .unwrap_or_else(|| {
                                status_object_missing_expr(status, property.clone())
                            })),
                        "Tag" => Ok(status_object_property_value(status, property)
                            .or_else(|| failure_virtual_tag_value(status))
                            .unwrap_or_else(|| {
                                status_object_missing_expr(status, property.clone())
                            })),
                        _ => Ok(
                            status_object_property_value(status, property).unwrap_or_else(|| {
                                status_object_missing_expr(status, property.clone())
                            }),
                        ),
                    };
                }
            } else if matches!(property, Expr::Atom(Atom::String(name)) if name == "Properties") {
                return Ok(status_object_properties_value(status)
                    .unwrap_or_else(|| status_object_missing_expr(status, property.clone())));
            }
        }

        Ok(
            status_object_property_value(status, property).unwrap_or_else(|| {
                let missing_property = if is_failure_expr(status) {
                    missing_property_context.unwrap_or(property)
                } else {
                    property
                };
                status_object_missing_expr(status, missing_property.clone())
            }),
        )
    }

    fn failure_virtual_message_value(&mut self, status: &Expr) -> Result<Option<Expr>, EvalError> {
        if let Some(message) = self.failure_message_from_template(status)? {
            return Ok(Some(message));
        }

        let Some((tag, _)) = status_object_parts(status) else {
            return Ok(None);
        };
        Ok(match tag {
            Expr::Atom(Atom::String(tag)) => Some(Expr::string(format!(
                "A failure of type \"{tag}\" occurred."
            ))),
            Expr::Atom(Atom::Symbol(_)) => Some(Expr::string("An unknown failure occurred.")),
            _ => None,
        })
    }

    fn failure_virtual_styled_message_value(
        &mut self,
        status: &Expr,
    ) -> Result<Option<Expr>, EvalError> {
        match self.failure_virtual_message_value(status) {
            Ok(Some(message)) => Ok(Some(styled_message_expr(message))),
            Ok(None) => Ok(None),
            Err(EvalError::InputRequired { request }) => {
                crate::resume_or_suspend!(insert self, FailureVirtualStyledMessageValueSuspensionFrame);
                Err(EvalError::InputRequired { request })
            }
            Err(error) => Err(error),
        }
    }

    fn failure_message_from_template(&mut self, status: &Expr) -> Result<Option<Expr>, EvalError> {
        let Some(template) = status_object_property_value(status, &Expr::string("MessageTemplate"))
        else {
            return Ok(None);
        };

        let Some((param_value, delayed)) = failure_message_template_parameter_value(status) else {
            return self
                .finish_failure_message_from_template(
                    status.clone(),
                    template,
                    FailureMessageTemplateParameters::default(),
                )
                .map(Some);
        };
        if delayed {
            let status = status.clone();
            let param_result = self.evaluate(param_value);
            let param_value = crate::resume_or_suspend!(
                self,
                param_result,
                ok(value) => Ok(value),
                suspend => {
            let status = status.clone();
                            let template = template.clone();
            FailureMessageFromTemplateSuspensionFrame { status, template }
            },
            )?;
            let params = failure_message_template_parameters_from_expr(&param_value);
            return self
                .finish_failure_message_from_template(status, template, params)
                .map(Some);
        }

        let params = failure_message_template_parameters_from_expr(&param_value);
        self.finish_failure_message_from_template(status.clone(), template, params)
            .map(Some)
    }

    fn finish_failure_message_from_template(
        &mut self,
        status: Expr,
        template: Expr,
        params: FailureMessageTemplateParameters,
    ) -> Result<Expr, EvalError> {
        if let Expr::Atom(Atom::String(template)) = template {
            let Some(key) = failure_message_template_key(&status) else {
                return Ok(Expr::string(template));
            };
            return Ok(Expr::string(format_message_template_with_named(
                &template,
                &key,
                &params.positional,
                &params.named,
            )));
        }

        let Some(key) = message_name_key_from_expr(&template) else {
            return Ok(template);
        };
        let rendered = self.render_message_text_with_named(&key, &params.positional, &params.named);
        crate::resume_or_suspend!(
            self,
            rendered,
            ok(text) => { (|| { Ok(Expr::string(text)) })() },
            suspend => FinishFailureMessageFromTemplateSuspensionFrame,
        )
    }
}

pub(in crate::runtime) fn is_failure_expr(expr: &Expr) -> bool {
    matches!(expr, Expr::Call { head, args, .. } if is_system_symbol_name(head, "Failure") && args.len() == 2)
}

pub(in crate::runtime) fn is_success_expr(expr: &Expr) -> bool {
    matches!(expr, Expr::Call { head, args, .. } if is_system_symbol_name(head, "Success") && args.len() == 2)
}

pub(in crate::runtime) fn is_failure_q_expr(expr: &Expr) -> bool {
    matches!(expr, Expr::Call { head, .. } if is_system_symbol_name(head, "Failure"))
        || is_system_symbol_name(expr, "$Failed")
        || is_system_symbol_name(expr, "$Aborted")
}

fn normalize_success_details_expr(expr: &Expr) -> Option<Expr> {
    if association_entries_from_expr(expr).is_some() {
        return Some(expr.clone());
    }

    None
}

fn normalize_failure_details_expr(expr: &Expr) -> Option<Expr> {
    if association_entries_from_expr(expr).is_some() {
        return Some(expr.clone());
    }

    if let Some(entry) = parse_association_entry_form(expr) {
        return Some(render_association_entry_forms_preserving_heads(vec![entry]));
    }

    let Expr::Call { head, args, .. } = expr else {
        return None;
    };

    if !is_system_symbol_name(head, heads::structure::LIST) {
        return None;
    }

    let mut entries = Vec::with_capacity(args.len());
    for item in args {
        entries.push(parse_immediate_rule_entry(item)?);
    }

    Some(render_association(entries))
}

fn parse_immediate_rule_entry(expr: &Expr) -> Option<(Expr, Expr)> {
    let Expr::Call { head, args, .. } = expr else {
        return None;
    };

    if !is_system_symbol_name(head, "Rule") || args.len() != 2 {
        return None;
    }

    Some((args[0].clone(), args[1].clone()))
}

fn status_object_properties_value(status: &Expr) -> Option<Expr> {
    let (_, details) = status_object_parts(status)?;
    let properties = association_entries_from_expr(details)?
        .into_iter()
        .map(|(key, _)| key)
        .collect::<Vec<_>>();
    Some(Expr::call(system_expr("List"), properties))
}

fn failure_properties_value(status: &Expr) -> Option<Expr> {
    let (_, details) = status_object_parts(status)?;
    let mut properties = association_entries_from_expr(details)?
        .into_iter()
        .map(|(key, _)| key)
        .collect::<Vec<_>>();
    if failure_virtual_held_message_template_value(status).is_some()
        || failure_virtual_message_name_value(status).is_some()
    {
        properties.push(Expr::string("HeldMessageTemplate"));
        properties.push(Expr::string("MessageName"));
    }
    properties.push(Expr::string("Message"));
    properties.push(Expr::string("StyledMessage"));
    properties.sort_by(compare_expr_canonical);
    properties.dedup();
    Some(Expr::call(system_expr("List"), properties))
}

fn status_object_all_value(status: &Expr) -> Option<Expr> {
    let (_, details) = status_object_parts(status)?;
    Some(details.clone())
}

fn status_object_property_value(status: &Expr, property: &Expr) -> Option<Expr> {
    let (_, details) = status_object_parts(status)?;
    association_value_by_key(details, property)
}

fn status_object_property_entry_form(
    status: &Expr,
    property: &Expr,
) -> Option<AssociationEntryForm> {
    let (_, details) = status_object_parts(status)?;
    association_entry_forms_from_expr(details)?
        .into_iter()
        .find(|entry| entry.key == *property)
}

fn failure_virtual_held_message_template_value(status: &Expr) -> Option<Expr> {
    let template = status_object_property_value(status, &Expr::string("MessageTemplate"))?;
    if !is_failure_message_template_virtualizable(&template) {
        return None;
    }

    Some(Expr::call(system_expr("Hold"), vec![template]))
}

fn failure_virtual_message_name_value(status: &Expr) -> Option<Expr> {
    let template = status_object_property_value(status, &Expr::string("MessageTemplate"))?;
    if let Some(key) = message_name_key_from_expr(&template) {
        return Some(Expr::string(key.tag));
    }

    if matches!(template, Expr::Atom(Atom::String(_))) {
        return Some(Expr::call(
            system_expr("Missing"),
            vec![Expr::string("NotApplicable")],
        ));
    }

    None
}

fn is_failure_message_template_virtualizable(template: &Expr) -> bool {
    matches!(template, Expr::Atom(Atom::String(_)))
        || message_name_key_from_expr(template).is_some()
}

fn failure_virtual_tag_value(status: &Expr) -> Option<Expr> {
    let (tag, details) = status_object_parts(status)?;
    association_entries_from_expr(details)?;
    match tag {
        Expr::Atom(Atom::String(tag)) => Some(Expr::string(tag.clone())),
        Expr::Atom(Atom::Symbol(tag)) => Some(Expr::Atom(Atom::Symbol(tag.clone()))),
        _ => None,
    }
}

fn styled_message_expr(message: Expr) -> Expr {
    Expr::call(system_expr("Style"), vec![message, Expr::string("Message")])
}

fn failure_message_template_key(status: &Expr) -> Option<MessageNameKey> {
    let (tag, _) = status_object_parts(status)?;
    Some(MessageNameKey {
        symbol: Symbol::system("Failure"),
        tag: match tag {
            Expr::Atom(Atom::String(tag)) => tag.clone(),
            Expr::Atom(Atom::Symbol(tag)) => tag.name().to_string(),
            _ => return None,
        },
    })
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
struct FailureMessageTemplateParameters {
    positional: Vec<Expr>,
    named: Vec<(String, Expr)>,
}

fn failure_message_template_parameter_value(status: &Expr) -> Option<(Expr, bool)> {
    let entry = status_object_property_entry_form(status, &Expr::string("MessageParameters"))?;
    Some((entry.value, entry.delayed))
}

fn failure_message_template_parameters_from_expr(expr: &Expr) -> FailureMessageTemplateParameters {
    if let Some(items) = list_items(expr) {
        return FailureMessageTemplateParameters {
            positional: items.to_vec(),
            named: Vec::new(),
        };
    }

    let Some(entries) = association_entries_from_expr(expr) else {
        return FailureMessageTemplateParameters::default();
    };

    FailureMessageTemplateParameters {
        positional: Vec::new(),
        named: entries
            .into_iter()
            .filter_map(|(key, value)| Some((message_parameter_name(&key)?, value)))
            .collect(),
    }
}

fn message_parameter_name(key: &Expr) -> Option<String> {
    match key {
        Expr::Atom(Atom::String(name)) => Some(name.clone()),
        Expr::Atom(Atom::Symbol(symbol)) => Some(symbol.name().to_string()),
        _ => None,
    }
}

fn status_object_missing_expr(status: &Expr, property: Expr) -> Expr {
    Expr::call(
        system_expr("Missing"),
        vec![
            Expr::string(if is_failure_expr(status) {
                "NotAvailable"
            } else {
                "KeyAbsent"
            }),
            property,
        ],
    )
}

fn status_object_parts(status: &Expr) -> Option<(&Expr, &Expr)> {
    let Expr::Call { head, args, .. } = status else {
        return None;
    };

    if !(is_system_symbol_name(head, "Failure") || is_system_symbol_name(head, "Success")) {
        return None;
    }

    let [tag, details] = args.as_slice() else {
        return None;
    };

    Some((tag, details))
}

fn failure_tag_supports_projection(status: &Expr) -> bool {
    let Some((tag, details)) = status_object_parts(status) else {
        return false;
    };

    association_entries_from_expr(details).is_some()
        && matches!(tag, Expr::Atom(Atom::String(_) | Atom::Symbol(_)))
}

fn success_tag_supports_projection(status: &Expr) -> bool {
    let Some((tag, details)) = status_object_parts(status) else {
        return false;
    };

    association_entries_from_expr(details).is_some() && matches!(tag, Expr::Atom(Atom::String(_)))
}
