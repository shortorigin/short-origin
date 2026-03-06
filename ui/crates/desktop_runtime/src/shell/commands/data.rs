use std::{cmp::Ordering, rc::Rc};

use desktop_app_contract::AppCommandRegistration;
use system_shell_contract::{
    CommandArgSpec, CommandDataShape, CommandOutputShape, DisplayPreference, StructuredData,
    StructuredRecord, StructuredScalar, StructuredValue,
};

pub(super) fn registrations() -> Vec<AppCommandRegistration> {
    vec![
        data_select_registration(),
        data_where_registration(),
        data_sort_registration(),
        data_first_registration(),
        data_get_registration(),
    ]
}

fn data_select_registration() -> AppCommandRegistration {
    AppCommandRegistration {
        descriptor: super::super::namespaced_descriptor(
            "data select",
            &[],
            "Select a subset of fields from records or tables.",
            "data select <field...>",
            vec![CommandArgSpec {
                name: "field".to_string(),
                summary: "Field names to keep.".to_string(),
                required: true,
                repeatable: true,
            }],
            Vec::new(),
            system_shell_contract::CommandInputShape::accepts(CommandDataShape::Any),
            CommandOutputShape::new(CommandDataShape::Any),
        ),
        completion: None,
        handler: Rc::new(|context| {
            Box::pin(async move {
                if context.args.is_empty() {
                    return Err(super::super::usage_error("usage: data select <field...>"));
                }
                match &context.input {
                    StructuredData::Table(table) => {
                        let rows = table
                            .rows
                            .iter()
                            .map(|row| StructuredRecord {
                                fields: context
                                    .args
                                    .iter()
                                    .filter_map(|name| {
                                        row.fields.iter().find(|field| &field.name == name).cloned()
                                    })
                                    .collect(),
                            })
                            .collect();
                        Ok(system_shell_contract::CommandResult {
                            output: super::super::table_data(
                                context.args.clone(),
                                rows,
                                Some(system_shell_contract::CommandPath::new("data select")),
                            ),
                            display: DisplayPreference::Table,
                            notices: Vec::new(),
                            cwd: None,
                            exit: system_shell_contract::ShellExit::success(),
                        })
                    }
                    StructuredData::Record(record) => Ok(system_shell_contract::CommandResult {
                        output: StructuredData::Record(StructuredRecord {
                            fields: context
                                .args
                                .iter()
                                .filter_map(|name| {
                                    record
                                        .fields
                                        .iter()
                                        .find(|field| &field.name == name)
                                        .cloned()
                                })
                                .collect(),
                        }),
                        display: DisplayPreference::Record,
                        notices: Vec::new(),
                        cwd: None,
                        exit: system_shell_contract::ShellExit::success(),
                    }),
                    StructuredData::Value(StructuredValue::Record(record)) => {
                        Ok(system_shell_contract::CommandResult {
                            output: StructuredData::Record(StructuredRecord {
                                fields: context
                                    .args
                                    .iter()
                                    .filter_map(|name| {
                                        record
                                            .fields
                                            .iter()
                                            .find(|field| &field.name == name)
                                            .cloned()
                                    })
                                    .collect(),
                            }),
                            display: DisplayPreference::Record,
                            notices: Vec::new(),
                            cwd: None,
                            exit: system_shell_contract::ShellExit::success(),
                        })
                    }
                    _ => Err(super::super::usage_error(
                        "data select expects record or table input",
                    )),
                }
            })
        }),
    }
}

fn data_sort_registration() -> AppCommandRegistration {
    AppCommandRegistration {
        descriptor: super::super::namespaced_descriptor(
            "data sort",
            &[],
            "Sort table rows by a field.",
            "data sort <field> [--desc]",
            vec![CommandArgSpec {
                name: "field".to_string(),
                summary: "Field to sort by.".to_string(),
                required: true,
                repeatable: false,
            }],
            Vec::new(),
            system_shell_contract::CommandInputShape::accepts(CommandDataShape::Table),
            CommandOutputShape::new(CommandDataShape::Table),
        ),
        completion: None,
        handler: Rc::new(|context| {
            Box::pin(async move {
                let field = context
                    .args
                    .first()
                    .ok_or_else(|| super::super::usage_error("usage: data sort <field> [--desc]"))?
                    .clone();
                let mut table = super::super::data_table_input(&context)?;
                let descending = context
                    .invocation
                    .options
                    .iter()
                    .any(|option| option.name == "desc");
                table.rows.sort_by(|left, right| {
                    let left_value = super::super::field_value(left, &field);
                    let right_value = super::super::field_value(right, &field);
                    let ord = match (left_value, right_value) {
                        (Some(left), Some(right)) => super::super::compare_scalar(left, right),
                        (Some(_), None) => Ordering::Greater,
                        (None, Some(_)) => Ordering::Less,
                        (None, None) => Ordering::Equal,
                    };
                    if descending {
                        ord.reverse()
                    } else {
                        ord
                    }
                });
                Ok(system_shell_contract::CommandResult {
                    output: StructuredData::Table(table),
                    display: DisplayPreference::Table,
                    notices: Vec::new(),
                    cwd: None,
                    exit: system_shell_contract::ShellExit::success(),
                })
            })
        }),
    }
}

fn data_where_registration() -> AppCommandRegistration {
    AppCommandRegistration {
        descriptor: super::super::namespaced_descriptor(
            "data where",
            &[],
            "Filter table rows by a field predicate.",
            "data where <field> <op> <value>",
            vec![
                CommandArgSpec {
                    name: "field".to_string(),
                    summary: "Field to inspect.".to_string(),
                    required: true,
                    repeatable: false,
                },
                CommandArgSpec {
                    name: "op".to_string(),
                    summary: "Predicate operator.".to_string(),
                    required: true,
                    repeatable: false,
                },
                CommandArgSpec {
                    name: "value".to_string(),
                    summary: "Expected value.".to_string(),
                    required: true,
                    repeatable: false,
                },
            ],
            Vec::new(),
            system_shell_contract::CommandInputShape::accepts(CommandDataShape::Table),
            CommandOutputShape::new(CommandDataShape::Table),
        ),
        completion: None,
        handler: Rc::new(|context| {
            Box::pin(async move {
                if context.args.len() < 3 {
                    return Err(super::super::usage_error(
                        "usage: data where <field> <op> <value>",
                    ));
                }
                let field = &context.args[0];
                let op = &context.args[1];
                let expected = context
                    .invocation
                    .values
                    .get(2)
                    .map(super::super::parsed_value_to_structured)
                    .unwrap_or_else(|| {
                        StructuredValue::Scalar(StructuredScalar::String(context.args[2].clone()))
                    });
                let table = super::super::data_table_input(&context)?;
                let rows = table
                    .rows
                    .into_iter()
                    .filter(|row| {
                        super::super::field_value(row, field)
                            .map(|value| super::super::predicate_matches(value, op, &expected))
                            .unwrap_or(false)
                    })
                    .collect();
                Ok(system_shell_contract::CommandResult {
                    output: super::super::table_data(
                        table.columns,
                        rows,
                        Some(system_shell_contract::CommandPath::new("data where")),
                    ),
                    display: DisplayPreference::Table,
                    notices: Vec::new(),
                    cwd: None,
                    exit: system_shell_contract::ShellExit::success(),
                })
            })
        }),
    }
}

fn data_first_registration() -> AppCommandRegistration {
    AppCommandRegistration {
        descriptor: super::super::namespaced_descriptor(
            "data first",
            &[],
            "Take the first row or rows from table/list input.",
            "data first [count]",
            vec![CommandArgSpec {
                name: "count".to_string(),
                summary: "Number of items to keep.".to_string(),
                required: false,
                repeatable: false,
            }],
            Vec::new(),
            system_shell_contract::CommandInputShape::accepts(CommandDataShape::Any),
            CommandOutputShape::new(CommandDataShape::Any),
        ),
        completion: None,
        handler: Rc::new(|context| {
            Box::pin(async move {
                let count = context
                    .args
                    .first()
                    .and_then(|value| value.parse::<usize>().ok())
                    .unwrap_or(1);
                match &context.input {
                    StructuredData::Table(table) => Ok(system_shell_contract::CommandResult {
                        output: super::super::table_data(
                            table.columns.clone(),
                            table.rows.iter().take(count).cloned().collect(),
                            Some(system_shell_contract::CommandPath::new("data first")),
                        ),
                        display: DisplayPreference::Table,
                        notices: Vec::new(),
                        cwd: None,
                        exit: system_shell_contract::ShellExit::success(),
                    }),
                    StructuredData::List(values) => Ok(system_shell_contract::CommandResult {
                        output: StructuredData::List(values.iter().take(count).cloned().collect()),
                        display: DisplayPreference::Value,
                        notices: Vec::new(),
                        cwd: None,
                        exit: system_shell_contract::ShellExit::success(),
                    }),
                    _ => Err(super::super::usage_error(
                        "data first expects table or list input",
                    )),
                }
            })
        }),
    }
}

fn data_get_registration() -> AppCommandRegistration {
    AppCommandRegistration {
        descriptor: super::super::namespaced_descriptor(
            "data get",
            &[],
            "Extract one field from record or table input.",
            "data get <field>",
            vec![CommandArgSpec {
                name: "field".to_string(),
                summary: "Field name.".to_string(),
                required: true,
                repeatable: false,
            }],
            Vec::new(),
            system_shell_contract::CommandInputShape::accepts(CommandDataShape::Any),
            CommandOutputShape::new(CommandDataShape::Any),
        ),
        completion: None,
        handler: Rc::new(|context| {
            Box::pin(async move {
                let field = context
                    .args
                    .first()
                    .ok_or_else(|| super::super::usage_error("usage: data get <field>"))?;
                match &context.input {
                    StructuredData::Table(table) => Ok(system_shell_contract::CommandResult {
                        output: StructuredData::List(
                            table
                                .rows
                                .iter()
                                .filter_map(|row| super::super::field_value(row, field).cloned())
                                .collect(),
                        ),
                        display: DisplayPreference::Value,
                        notices: Vec::new(),
                        cwd: None,
                        exit: system_shell_contract::ShellExit::success(),
                    }),
                    StructuredData::Record(record) => {
                        let value = super::super::field_value(record, field)
                            .cloned()
                            .ok_or_else(|| {
                                super::super::usage_error(format!("missing field `{field}`"))
                            })?;
                        Ok(system_shell_contract::CommandResult {
                            output: StructuredData::Value(value),
                            display: DisplayPreference::Value,
                            notices: Vec::new(),
                            cwd: None,
                            exit: system_shell_contract::ShellExit::success(),
                        })
                    }
                    StructuredData::Value(StructuredValue::Record(record)) => {
                        let value = super::super::field_value(record, field)
                            .cloned()
                            .ok_or_else(|| {
                                super::super::usage_error(format!("missing field `{field}`"))
                            })?;
                        Ok(system_shell_contract::CommandResult {
                            output: StructuredData::Value(value),
                            display: DisplayPreference::Value,
                            notices: Vec::new(),
                            cwd: None,
                            exit: system_shell_contract::ShellExit::success(),
                        })
                    }
                    _ => Err(super::super::usage_error(
                        "data get expects record or table input",
                    )),
                }
            })
        }),
    }
}
