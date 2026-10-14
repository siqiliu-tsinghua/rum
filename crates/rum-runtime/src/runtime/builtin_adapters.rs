mod resume_frames;
use resume_frames::*;

use super::*;

pub(in crate::runtime) fn builtin_identity(
    _session: &mut Session,
    head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    Ok(if args.len() == 1 {
        args[0].clone()
    } else {
        Expr::call(head.clone(), args.to_vec())
    })
}

pub(in crate::runtime) fn builtin_trace(
    session: &mut Session,
    _head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    session.eval_trace(args)
}

pub(in crate::runtime) fn builtin_trace_detail(
    session: &mut Session,
    _head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    session.eval_trace_detail(args)
}

pub(in crate::runtime) fn builtin_trace_print(
    session: &mut Session,
    _head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    session.eval_trace_print(args)
}

pub(in crate::runtime) fn builtin_timing(
    session: &mut Session,
    _head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    session.eval_timing(args)
}

pub(in crate::runtime) fn builtin_absolute_timing(
    session: &mut Session,
    _head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    session.eval_absolute_timing(args)
}

pub(in crate::runtime) fn builtin_repeated_timing(
    session: &mut Session,
    _head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    session.eval_repeated_timing(args)
}

pub(in crate::runtime) fn builtin_on(
    session: &mut Session,
    _head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    session.eval_on(args)
}

pub(in crate::runtime) fn builtin_off(
    session: &mut Session,
    _head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    session.eval_off(args)
}

pub(in crate::runtime) fn builtin_reap(
    session: &mut Session,
    _head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    session.eval_reap(args)
}

pub(in crate::runtime) fn builtin_time_constrained(
    session: &mut Session,
    _head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    session.eval_time_constrained(args)
}

pub(in crate::runtime) fn builtin_memory_constrained(
    session: &mut Session,
    _head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    session.eval_memory_constrained(args)
}

pub(in crate::runtime) fn builtin_byte_count(
    session: &mut Session,
    head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    session.eval_byte_count(head, args.as_ref())
}

pub(in crate::runtime) fn builtin_memory_in_use(
    session: &mut Session,
    head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    session.eval_memory_in_use(head, args.as_ref())
}

pub(in crate::runtime) fn builtin_max_memory_used(
    session: &mut Session,
    head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    session.eval_max_memory_used(head, args.as_ref())
}

pub(in crate::runtime) fn builtin_monitor(
    session: &mut Session,
    _head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    session.eval_monitor(args)
}

pub(in crate::runtime) fn builtin_evaluation_data(
    session: &mut Session,
    _head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    session.eval_evaluation_data(args)
}

pub(in crate::runtime) fn builtin_rum_profile_reset(
    session: &mut Session,
    _head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    session.eval_rum_profile_reset(args)
}

pub(in crate::runtime) fn builtin_rum_profile_snapshot(
    session: &mut Session,
    _head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    session.eval_rum_profile_snapshot(args)
}

pub(in crate::runtime) fn builtin_stack(
    session: &mut Session,
    _head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    session.eval_stack(args)
}

pub(in crate::runtime) fn builtin_set(
    session: &mut Session,
    _head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    session.eval_set(args)
}

pub(in crate::runtime) fn builtin_add_to(
    session: &mut Session,
    _head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    session.eval_add_to(args)
}

pub(in crate::runtime) fn builtin_subtract_from(
    session: &mut Session,
    _head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    session.eval_subtract_from(args)
}

pub(in crate::runtime) fn builtin_times_by(
    session: &mut Session,
    _head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    session.eval_times_by(args)
}

pub(in crate::runtime) fn builtin_divide_by(
    session: &mut Session,
    _head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    session.eval_divide_by(args)
}

pub(in crate::runtime) fn builtin_apply_to(
    session: &mut Session,
    _head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    session.eval_apply_to(args)
}

pub(in crate::runtime) fn builtin_append_to(
    session: &mut Session,
    _head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    session.eval_append_to(args)
}

pub(in crate::runtime) fn builtin_prepend_to(
    session: &mut Session,
    _head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    session.eval_prepend_to(args)
}

pub(in crate::runtime) fn builtin_increment(
    session: &mut Session,
    _head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    session.eval_increment(args)
}

pub(in crate::runtime) fn builtin_decrement(
    session: &mut Session,
    _head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    session.eval_decrement(args)
}

pub(in crate::runtime) fn builtin_pre_increment(
    session: &mut Session,
    _head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    session.eval_pre_increment(args)
}

pub(in crate::runtime) fn builtin_pre_decrement(
    session: &mut Session,
    _head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    session.eval_pre_decrement(args)
}

pub(in crate::runtime) fn builtin_set_delayed(
    session: &mut Session,
    _head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    session.eval_set_delayed(args)
}

pub(in crate::runtime) fn builtin_up_set(
    session: &mut Session,
    _head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    session.eval_up_set(args)
}

pub(in crate::runtime) fn builtin_up_set_delayed(
    session: &mut Session,
    _head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    session.eval_up_set_delayed(args)
}

pub(in crate::runtime) fn builtin_tag_set(
    session: &mut Session,
    _head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    session.eval_tag_set(args)
}

pub(in crate::runtime) fn builtin_tag_set_delayed(
    session: &mut Session,
    _head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    session.eval_tag_set_delayed(args)
}

pub(in crate::runtime) fn builtin_tag_unset(
    session: &mut Session,
    _head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    session.eval_tag_unset(args)
}

pub(in crate::runtime) fn builtin_rule(
    session: &mut Session,
    _head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    session.eval_rule(args)
}

pub(in crate::runtime) fn builtin_rule_delayed(
    session: &mut Session,
    _head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    session.eval_rule_delayed(args)
}

pub(in crate::runtime) fn builtin_count(
    session: &mut Session,
    _head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    session.eval_count(args)
}

pub(in crate::runtime) fn builtin_member_q(
    session: &mut Session,
    _head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    session.eval_member_q(args)
}

pub(in crate::runtime) fn builtin_free_q(
    session: &mut Session,
    _head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    session.eval_free_q(args)
}

pub(in crate::runtime) fn builtin_position(
    session: &mut Session,
    _head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    session.eval_position(args)
}

pub(in crate::runtime) fn builtin_module(
    session: &mut Session,
    _head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    session.eval_module(args)
}

pub(in crate::runtime) fn builtin_define_package(
    session: &mut Session,
    _head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    session.eval_define_package(args)
}

pub(in crate::runtime) fn builtin_restore_sealed_references(
    session: &mut Session,
    _head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    session.eval_restore_sealed_references(args)
}

pub(in crate::runtime) fn builtin_public(
    _session: &mut Session,
    head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    Ok(Expr::call(head.clone(), args.to_vec()))
}

pub(in crate::runtime) fn builtin_package_scope(
    _session: &mut Session,
    head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    Ok(Expr::call(head.clone(), args.to_vec()))
}

pub(in crate::runtime) fn builtin_unique(
    session: &mut Session,
    _head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    session.eval_unique(args)
}

pub(in crate::runtime) fn builtin_block(
    session: &mut Session,
    _head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    session.eval_block(args)
}

pub(in crate::runtime) fn builtin_find_file(
    session: &mut Session,
    _head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    session.eval_find_file(args)
}

pub(in crate::runtime) fn builtin_file_name_join(
    session: &mut Session,
    _head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    session.eval_file_name_join(args)
}

pub(in crate::runtime) fn builtin_directory_name(
    session: &mut Session,
    _head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    session.eval_directory_name(args)
}

pub(in crate::runtime) fn builtin_parent_directory(
    session: &mut Session,
    _head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    session.eval_parent_directory(args)
}

pub(in crate::runtime) fn builtin_file_name_take(
    session: &mut Session,
    _head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    session.eval_file_name_take(args)
}

pub(in crate::runtime) fn builtin_get(
    session: &mut Session,
    _head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    session.eval_get(args)
}

pub(in crate::runtime) fn builtin_needs(
    session: &mut Session,
    _head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    session.eval_needs(args)
}

pub(in crate::runtime) fn builtin_context(
    session: &mut Session,
    _head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    session.eval_context(args)
}

pub(in crate::runtime) fn builtin_contexts(
    session: &mut Session,
    _head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    session.eval_contexts(args)
}

pub(in crate::runtime) fn builtin_names(
    session: &mut Session,
    _head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    session.eval_names(args)
}

pub(in crate::runtime) fn builtin_begin(
    session: &mut Session,
    _head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    session.eval_begin(args)
}

pub(in crate::runtime) fn builtin_end(
    session: &mut Session,
    _head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    session.eval_end(args)
}

pub(in crate::runtime) fn builtin_begin_package(
    session: &mut Session,
    _head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    session.eval_begin_package(args)
}

pub(in crate::runtime) fn builtin_end_package(
    session: &mut Session,
    _head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    session.eval_end_package(args)
}

pub(in crate::runtime) fn builtin_declare_package(
    session: &mut Session,
    _head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    session.eval_declare_package(args)
}

pub(in crate::runtime) fn builtin_save(
    session: &mut Session,
    _head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    session.eval_save(args)
}

pub(in crate::runtime) fn builtin_dump_save(
    session: &mut Session,
    _head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    session.eval_dump_save(args)
}

pub(in crate::runtime) fn builtin_once(
    session: &mut Session,
    _head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    session.eval_once_builtin(args)
}

pub(in crate::runtime) fn builtin_message(
    session: &mut Session,
    _head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    session.eval_message(args)
}

pub(in crate::runtime) fn builtin_message_name(
    session: &mut Session,
    _head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    session.eval_message_name(args)
}

pub(in crate::runtime) fn builtin_messages(
    session: &mut Session,
    _head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    session.eval_messages(args)
}

pub(in crate::runtime) fn builtin_message_definitions(
    session: &mut Session,
    _head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    session.eval_message_definitions(args)
}

pub(in crate::runtime) fn builtin_message_tags(
    session: &mut Session,
    _head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    session.eval_message_tags(args)
}

pub(in crate::runtime) fn builtin_definition(
    session: &mut Session,
    _head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    session.eval_definition(args)
}

pub(in crate::runtime) fn builtin_value_q(
    session: &mut Session,
    _head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    session.eval_value_q(args)
}

pub(in crate::runtime) fn builtin_default_values(
    session: &mut Session,
    _head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    session.eval_default_values(args)
}

pub(in crate::runtime) fn builtin_own_values(
    session: &mut Session,
    _head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    session.eval_own_values(args)
}

pub(in crate::runtime) fn builtin_up_values(
    session: &mut Session,
    _head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    session.eval_up_values(args)
}

pub(in crate::runtime) fn builtin_down_values(
    session: &mut Session,
    _head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    session.eval_down_values(args)
}

pub(in crate::runtime) fn builtin_sub_values(
    session: &mut Session,
    _head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    session.eval_sub_values(args)
}

pub(in crate::runtime) fn builtin_attributes(
    session: &mut Session,
    _head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    session.eval_attributes(args)
}

pub(in crate::runtime) fn builtin_syntax_information(
    session: &mut Session,
    _head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    session.eval_syntax_information(args)
}

pub(in crate::runtime) fn builtin_options(
    session: &mut Session,
    _head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    session.eval_options(args)
}

pub(in crate::runtime) fn builtin_option_value(
    session: &mut Session,
    _head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    session.eval_option_value(args)
}

pub(in crate::runtime) fn builtin_arguments_options(
    session: &mut Session,
    _head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    session.eval_arguments_options(args)
}

pub(in crate::runtime) fn builtin_check_arguments(
    session: &mut Session,
    _head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    session.eval_check_arguments(args)
}

pub(in crate::runtime) fn builtin_set_attributes(
    session: &mut Session,
    _head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    session.eval_set_attributes(args)
}

pub(in crate::runtime) fn builtin_clear_attributes(
    session: &mut Session,
    _head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    session.eval_clear_attributes(args)
}

pub(in crate::runtime) fn builtin_set_options(
    session: &mut Session,
    _head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    session.eval_set_options(args)
}

pub(in crate::runtime) fn builtin_protect(
    session: &mut Session,
    _head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    session.eval_protect(args)
}

pub(in crate::runtime) fn builtin_unprotect(
    session: &mut Session,
    _head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    session.eval_unprotect(args)
}

pub(in crate::runtime) fn builtin_unset(
    session: &mut Session,
    _head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    session.eval_unset(args)
}

pub(in crate::runtime) fn builtin_clear(
    session: &mut Session,
    _head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    session.eval_clear(args)
}

pub(in crate::runtime) fn builtin_clear_all(
    session: &mut Session,
    _head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    session.eval_clear_all(args)
}

pub(in crate::runtime) fn builtin_remove(
    session: &mut Session,
    _head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    session.eval_remove(args)
}

pub(in crate::runtime) fn builtin_normalize_rules(
    session: &mut Session,
    _head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    session.eval_normalize_rules(args)
}

pub(in crate::runtime) fn builtin_check(
    session: &mut Session,
    _head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    session.eval_check(args)
}

pub(in crate::runtime) fn builtin_quiet(
    session: &mut Session,
    _head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    session.eval_quiet(args)
}

pub(in crate::runtime) fn builtin_with(
    session: &mut Session,
    _head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    session.eval_with(args)
}

pub(in crate::runtime) fn builtin_hold_literal(
    _session: &mut Session,
    head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    Ok(Expr::call(head.clone(), args.to_vec()))
}

pub(in crate::runtime) fn builtin_unevaluated(
    _session: &mut Session,
    head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    Ok(if args.len() == 1 {
        args[0].clone()
    } else {
        Expr::call(head.clone(), args.to_vec())
    })
}

pub(in crate::runtime) fn builtin_release_hold(
    session: &mut Session,
    _head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    session.eval_release_hold(args)
}

pub(in crate::runtime) fn builtin_evaluate(
    session: &mut Session,
    head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    Ok(if args.len() == 1 {
        let evaluated_result = session.evaluate(args[0].clone());
        crate::resume_typed_or_suspend!(
            session,
            evaluated_result,
            ok(value) => Ok(value),
            suspend => BuiltinEvaluateSuspensionFrame,
        )?
    } else {
        Expr::call(head.clone(), args.to_vec())
    })
}

pub(in crate::runtime) fn builtin_sequence(
    _session: &mut Session,
    _head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    Ok(sequence_expr(args.to_vec()))
}

pub(in crate::runtime) fn builtin_dialog(
    session: &mut Session,
    _head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    session.eval_dialog(args)
}

pub(in crate::runtime) fn builtin_input_string(
    session: &mut Session,
    _head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    session.eval_input_string(args)
}

pub(in crate::runtime) fn builtin_input(
    session: &mut Session,
    _head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    session.eval_input(args)
}

pub(in crate::runtime) fn builtin_print(
    session: &mut Session,
    _head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    session.eval_print(args)
}

pub(in crate::runtime) fn builtin_pause(
    session: &mut Session,
    _head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    session.eval_pause(args)
}
