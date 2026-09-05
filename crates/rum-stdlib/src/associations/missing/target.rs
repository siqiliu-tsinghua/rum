use super::*;

const RECURSIVE_DELETE_MISSING_TARGET_DEPTH: usize = 16;

struct OwnedListCursor {
    items: Vec<Expr>,
    level: usize,
    scan: DeleteMissingDepth,
    index: usize,
    kept: Vec<Expr>,
}

struct OwnedAssociationCursor {
    entries: Vec<AssociationEntryForm>,
    level: usize,
    scan: DeleteMissingDepth,
    index: usize,
    kept: Vec<AssociationEntryForm>,
}

enum OwnedTargetCursor {
    List(OwnedListCursor),
    Association(OwnedAssociationCursor),
}

pub(super) type FrozenListCursor = (
    FrozenAssociationSequence<Expr>,
    usize,
    DeleteMissingDepth,
    usize,
    PersistentAssociationSequence<Expr>,
);
pub(super) type FrozenAssociationCursor = (
    FrozenAssociationSequence<AssociationEntryForm>,
    usize,
    DeleteMissingDepth,
    usize,
    PersistentAssociationSequence<AssociationEntryForm>,
);
type FrozenTargetCursor = Result<FrozenListCursor, FrozenAssociationCursor>;
type FrozenAssociationFinish = Result<
    (FrozenAssociationCursor, AssociationEntryForm),
    (FrozenAssociationCursor, AssociationEntryForm),
>;
type FrozenTargetFinish = Result<FrozenListCursor, FrozenAssociationFinish>;
pub(super) type DeleteMissingTargetWork = Result<FrozenTargetCursor, FrozenTargetFinish>;

struct TargetContinuation {
    active: Vec<DeleteMissingTargetWork>,
    parents: Vec<DeleteMissingTargetWork>,
}

impl TargetContinuation {
    fn starting(active: DeleteMissingTargetWork) -> Self {
        Self {
            active: vec![active],
            parents: Vec::new(),
        }
    }

    fn push_parent(&mut self, parent: DeleteMissingTargetWork) {
        self.parents.push(parent);
    }

    #[cfg(test)]
    fn work_len(&self) -> usize {
        self.active.len() + self.parents.len()
    }

    fn into_work(mut self) -> Vec<DeleteMissingTargetWork> {
        self.parents.reverse();
        self.parents.append(&mut self.active);
        self.parents
    }
}

enum TargetTraversal {
    Complete(Expr),
    Exhausted(TargetContinuation),
}

enum TargetTraversalError {
    InputRequired {
        error: EvalError,
        continuation: TargetContinuation,
    },
    Evaluation(EvalError),
}

type TargetTraversalResult = Result<TargetTraversal, TargetTraversalError>;

pub(super) fn delete_missing_at_target_level(
    session: &mut Session,
    expr: &Expr,
    level: usize,
    scan: DeleteMissingDepth,
) -> Result<Option<Expr>, EvalError> {
    delete_missing_at_target_level_with_front_depth(
        session,
        expr,
        level,
        scan,
        RECURSIVE_DELETE_MISSING_TARGET_DEPTH,
    )
}

fn delete_missing_at_target_level_with_front_depth(
    session: &mut Session,
    expr: &Expr,
    level: usize,
    scan: DeleteMissingDepth,
    remaining_call_depth: usize,
) -> Result<Option<Expr>, EvalError> {
    let cursor = if let Some(items) = list_items(expr) {
        OwnedTargetCursor::List(OwnedListCursor {
            items: items.to_vec(),
            level,
            scan,
            index: 0,
            kept: Vec::new(),
        })
    } else if let Some(entries) = association_entry_forms_from_expr(expr) {
        OwnedTargetCursor::Association(OwnedAssociationCursor {
            entries,
            level,
            scan,
            index: 0,
            kept: Vec::new(),
        })
    } else {
        return Ok(None);
    };

    let traversal = drive_owned_target_cursor(session, cursor, remaining_call_depth);
    finish_bounded_target_traversal(session, traversal).map(Some)
}

fn drive_owned_target_cursor(
    session: &mut Session,
    cursor: OwnedTargetCursor,
    remaining_call_depth: usize,
) -> TargetTraversalResult {
    match cursor {
        OwnedTargetCursor::List(cursor) => {
            drive_owned_list_cursor(session, cursor, remaining_call_depth)
        }
        OwnedTargetCursor::Association(cursor) => {
            drive_owned_association_cursor(session, cursor, remaining_call_depth)
        }
    }
}

fn drive_owned_list_cursor(
    session: &mut Session,
    mut cursor: OwnedListCursor,
    remaining_call_depth: usize,
) -> TargetTraversalResult {
    loop {
        let Some(item) = cursor.items.get(cursor.index).cloned() else {
            return Ok(TargetTraversal::Complete(Expr::call(
                system_expr("List"),
                cursor.kept,
            )));
        };

        if cursor.level == 1 {
            if !rule_like_expr_has_missing_value(&item)
                && !contains_missing_within(&item, cursor.scan)
            {
                cursor.kept.push(item);
            }
            cursor.index += 1;
            continue;
        }

        let child = if let Some(items) = list_items(&item) {
            Some(OwnedTargetCursor::List(OwnedListCursor {
                items: items.to_vec(),
                level: cursor.level - 1,
                scan: cursor.scan,
                index: 0,
                kept: Vec::new(),
            }))
        } else {
            association_entry_forms_from_expr(&item).map(|entries| {
                OwnedTargetCursor::Association(OwnedAssociationCursor {
                    entries,
                    level: cursor.level - 1,
                    scan: cursor.scan,
                    index: 0,
                    kept: Vec::new(),
                })
            })
        };

        let Some(child) = child else {
            cursor.kept.push(item);
            cursor.index += 1;
            continue;
        };

        if remaining_call_depth == 0 {
            let active = freeze_owned_target_cursor(child);
            let parent = freeze_owned_list_cursor(cursor);
            let mut continuation = TargetContinuation::starting(active);
            continuation.push_parent(finish_list_child_work(parent));
            return Ok(TargetTraversal::Exhausted(continuation));
        }

        match drive_owned_target_cursor(session, child, remaining_call_depth - 1) {
            Ok(TargetTraversal::Complete(value)) => {
                cursor.kept.push(value);
                cursor.index += 1;
            }
            Ok(TargetTraversal::Exhausted(mut continuation)) => {
                continuation.push_parent(finish_list_child_work(freeze_owned_list_cursor(cursor)));
                return Ok(TargetTraversal::Exhausted(continuation));
            }
            Err(TargetTraversalError::InputRequired {
                error,
                mut continuation,
            }) => {
                continuation.push_parent(finish_list_child_work(freeze_owned_list_cursor(cursor)));
                return Err(TargetTraversalError::InputRequired {
                    error,
                    continuation,
                });
            }
            Err(error @ TargetTraversalError::Evaluation(_)) => return Err(error),
        }
    }
}

fn drive_owned_association_cursor(
    session: &mut Session,
    mut cursor: OwnedAssociationCursor,
    remaining_call_depth: usize,
) -> TargetTraversalResult {
    loop {
        let Some(mut entry) = cursor.entries.get(cursor.index).cloned() else {
            return Ok(TargetTraversal::Complete(
                render_association_entry_forms_preserving_heads(cursor.kept),
            ));
        };

        if is_missing_expr(&entry.value) {
            cursor.index += 1;
            continue;
        }

        if cursor.level == 1 {
            if !entry.delayed {
                if !contains_missing_within(&entry.value, cursor.scan) {
                    cursor.kept.push(entry);
                }
                cursor.index += 1;
                continue;
            }

            let predicate = entry.value.clone();
            match session.evaluate(predicate) {
                Ok(value) => {
                    if !is_missing_expr(&value) && !contains_missing_within(&value, cursor.scan) {
                        cursor.kept.push(entry);
                    }
                    cursor.index += 1;
                    continue;
                }
                Err(error @ EvalError::InputRequired { .. }) => {
                    let continuation =
                        TargetContinuation::starting(finish_association_predicate_work(
                            freeze_owned_association_cursor(cursor),
                            entry,
                        ));
                    return Err(TargetTraversalError::InputRequired {
                        error,
                        continuation,
                    });
                }
                Err(error) => return Err(TargetTraversalError::Evaluation(error)),
            }
        }

        if entry.delayed {
            entry.value = delete_missing_delayed_wrapper(
                entry.value,
                DeleteMissingMode::Target {
                    level: DeleteMissingDepth::Finite(cursor.level - 1),
                    scan: cursor.scan,
                },
            );
            cursor.kept.push(entry);
            cursor.index += 1;
            continue;
        }

        let child = if let Some(items) = list_items(&entry.value) {
            Some(OwnedTargetCursor::List(OwnedListCursor {
                items: items.to_vec(),
                level: cursor.level - 1,
                scan: cursor.scan,
                index: 0,
                kept: Vec::new(),
            }))
        } else {
            association_entry_forms_from_expr(&entry.value).map(|entries| {
                OwnedTargetCursor::Association(OwnedAssociationCursor {
                    entries,
                    level: cursor.level - 1,
                    scan: cursor.scan,
                    index: 0,
                    kept: Vec::new(),
                })
            })
        };

        let Some(child) = child else {
            cursor.kept.push(entry);
            cursor.index += 1;
            continue;
        };

        if remaining_call_depth == 0 {
            let active = freeze_owned_target_cursor(child);
            let parent = freeze_owned_association_cursor(cursor);
            let mut continuation = TargetContinuation::starting(active);
            continuation.push_parent(finish_association_child_work(parent, entry));
            return Ok(TargetTraversal::Exhausted(continuation));
        }

        match drive_owned_target_cursor(session, child, remaining_call_depth - 1) {
            Ok(TargetTraversal::Complete(value)) => {
                entry.value = value;
                cursor.kept.push(entry);
                cursor.index += 1;
            }
            Ok(TargetTraversal::Exhausted(mut continuation)) => {
                continuation.push_parent(finish_association_child_work(
                    freeze_owned_association_cursor(cursor),
                    entry,
                ));
                return Ok(TargetTraversal::Exhausted(continuation));
            }
            Err(TargetTraversalError::InputRequired {
                error,
                mut continuation,
            }) => {
                continuation.push_parent(finish_association_child_work(
                    freeze_owned_association_cursor(cursor),
                    entry,
                ));
                return Err(TargetTraversalError::InputRequired {
                    error,
                    continuation,
                });
            }
            Err(error @ TargetTraversalError::Evaluation(_)) => return Err(error),
        }
    }
}

// The exhausted handoff carries live cursors and completed prefixes rather
// than restarting from the root. That is the correct shape, but note WHY no
// test can currently observe a restarting one: delayed right-hand sides are
// evaluated only at `level == 1`, while front exhaustion can happen only at
// remaining front depth zero, which is still `level > 1`. So on the only path
// that has a front, exhaustion strictly precedes any user-code evaluation, and
// `resume_delete_missing_target_work` below drives entirely on the heap with no
// front at all - including after a suspension resumes into it. A restart would
// therefore redo pure structural work and be invisible.
//
// If a change ever lets user code run before an exhaustion - a predicate
// evaluated above target level, or a front reintroduced on the resume path -
// that invariant is gone and this handoff needs a real replay test.
fn finish_bounded_target_traversal(
    session: &mut Session,
    traversal: TargetTraversalResult,
) -> Result<Expr, EvalError> {
    match traversal {
        Ok(TargetTraversal::Complete(value)) => Ok(value),
        Ok(TargetTraversal::Exhausted(continuation)) => {
            resume_delete_missing_target_work(session, continuation.into_work(), None)
        }
        Err(TargetTraversalError::InputRequired {
            error,
            continuation,
        }) => {
            let work = FrozenAssociationSequence::freeze(continuation.into_work());
            rum_runtime::resume_or_suspend!(
                session,
                Err(error),
                ok(value) => Ok(value),
                suspend => ResumeDeleteMissingAssociationAtTargetLevelSuspensionFrame1 { work },
            )
        }
        Err(TargetTraversalError::Evaluation(error)) => Err(error),
    }
}

#[cold]
#[inline(never)]
fn resume_delete_missing_target_work(
    session: &mut Session,
    mut work: Vec<DeleteMissingTargetWork>,
    mut completed: Option<Expr>,
) -> Result<Expr, EvalError> {
    'work: while let Some(task) = work.pop() {
        match task {
            Ok(Ok((items, level, scan, mut index, mut kept))) => {
                debug_assert!(completed.is_none());
                loop {
                    let Some(item) = items.get(index).cloned() else {
                        completed = Some(Expr::call(system_expr("List"), kept.to_vec()));
                        continue 'work;
                    };

                    if level == 1 {
                        if !rule_like_expr_has_missing_value(&item)
                            && !contains_missing_within(&item, scan)
                        {
                            kept = kept.appended(item);
                        }
                        index += 1;
                        continue;
                    }

                    let child = if let Some(items) = list_items(&item) {
                        Some(drive_list_work((
                            FrozenAssociationSequence::freeze(items.to_vec()),
                            level - 1,
                            scan,
                            0,
                            PersistentAssociationSequence::freeze(Vec::new()),
                        )))
                    } else {
                        association_entry_forms_from_expr(&item).map(|entries| {
                            drive_association_work((
                                FrozenAssociationSequence::freeze(entries),
                                level - 1,
                                scan,
                                0,
                                PersistentAssociationSequence::freeze(Vec::new()),
                            ))
                        })
                    };

                    if let Some(child) = child {
                        work.push(finish_list_child_work((items, level, scan, index, kept)));
                        work.push(child);
                        continue 'work;
                    }

                    kept = kept.appended(item);
                    index += 1;
                }
            }
            Ok(Err((entries, level, scan, mut index, mut kept))) => {
                debug_assert!(completed.is_none());
                loop {
                    let Some(mut entry) = entries.get(index).cloned() else {
                        completed = Some(render_association_entry_forms_preserving_heads(
                            kept.to_vec(),
                        ));
                        continue 'work;
                    };

                    if is_missing_expr(&entry.value) {
                        index += 1;
                        continue;
                    }

                    if level == 1 {
                        if !entry.delayed {
                            if !contains_missing_within(&entry.value, scan) {
                                kept = kept.appended(entry);
                            }
                            index += 1;
                            continue;
                        }

                        let predicate = entry.value.clone();
                        work.push(finish_association_predicate_work(
                            (entries, level, scan, index, kept),
                            entry,
                        ));
                        match session.evaluate(predicate) {
                            Ok(value) => {
                                completed = Some(value);
                                continue 'work;
                            }
                            Err(EvalError::InputRequired { request }) => {
                                let work = FrozenAssociationSequence::freeze(work);
                                return rum_runtime::resume_or_suspend!(
                                    session,
                                    Err(EvalError::InputRequired { request }),
                                    ok(value) => Ok(value),
                                    suspend => ResumeDeleteMissingAssociationAtTargetLevelSuspensionFrame1 { work },
                                );
                            }
                            Err(error) => return Err(error),
                        }
                    }

                    if entry.delayed {
                        entry.value = delete_missing_delayed_wrapper(
                            entry.value,
                            DeleteMissingMode::Target {
                                level: DeleteMissingDepth::Finite(level - 1),
                                scan,
                            },
                        );
                        kept = kept.appended(entry);
                        index += 1;
                        continue;
                    }

                    let child = if let Some(items) = list_items(&entry.value) {
                        Some(drive_list_work((
                            FrozenAssociationSequence::freeze(items.to_vec()),
                            level - 1,
                            scan,
                            0,
                            PersistentAssociationSequence::freeze(Vec::new()),
                        )))
                    } else {
                        association_entry_forms_from_expr(&entry.value).map(|entries| {
                            drive_association_work((
                                FrozenAssociationSequence::freeze(entries),
                                level - 1,
                                scan,
                                0,
                                PersistentAssociationSequence::freeze(Vec::new()),
                            ))
                        })
                    };

                    if let Some(child) = child {
                        work.push(finish_association_child_work(
                            (entries, level, scan, index, kept),
                            entry,
                        ));
                        work.push(child);
                        continue 'work;
                    }

                    kept = kept.appended(entry);
                    index += 1;
                }
            }
            Err(Ok((items, level, scan, mut index, mut kept))) => {
                let value = completed
                    .take()
                    .expect("DeleteMissing target work completes a list child");
                kept = kept.appended(value);
                index += 1;
                work.push(drive_list_work((items, level, scan, index, kept)));
            }
            Err(Err(Ok(((entries, level, scan, mut index, mut kept), mut entry)))) => {
                entry.value = completed
                    .take()
                    .expect("DeleteMissing target work completes an association child");
                kept = kept.appended(entry);
                index += 1;
                work.push(drive_association_work((entries, level, scan, index, kept)));
            }
            Err(Err(Err(((entries, level, scan, mut index, mut kept), entry)))) => {
                let value = completed
                    .take()
                    .expect("DeleteMissing target work completes a delayed predicate");
                if !is_missing_expr(&value) && !contains_missing_within(&value, scan) {
                    kept = kept.appended(entry);
                }
                index += 1;
                work.push(drive_association_work((entries, level, scan, index, kept)));
            }
        }
    }

    Ok(completed.expect("DeleteMissing target work completes its root"))
}

pub(super) fn resume_suspended_delete_missing_target_work(
    session: &mut Session,
    work: FrozenAssociationSequence<DeleteMissingTargetWork>,
    value: Expr,
) -> Result<Expr, EvalError> {
    resume_delete_missing_target_work(session, work.as_slice().to_vec(), Some(value))
}

fn freeze_owned_target_cursor(cursor: OwnedTargetCursor) -> DeleteMissingTargetWork {
    match cursor {
        OwnedTargetCursor::List(cursor) => drive_list_work(freeze_owned_list_cursor(cursor)),
        OwnedTargetCursor::Association(cursor) => {
            drive_association_work(freeze_owned_association_cursor(cursor))
        }
    }
}

fn freeze_owned_list_cursor(cursor: OwnedListCursor) -> FrozenListCursor {
    (
        FrozenAssociationSequence::freeze(cursor.items),
        cursor.level,
        cursor.scan,
        cursor.index,
        PersistentAssociationSequence::freeze(cursor.kept),
    )
}

fn freeze_owned_association_cursor(cursor: OwnedAssociationCursor) -> FrozenAssociationCursor {
    (
        FrozenAssociationSequence::freeze(cursor.entries),
        cursor.level,
        cursor.scan,
        cursor.index,
        PersistentAssociationSequence::freeze(cursor.kept),
    )
}

fn drive_list_work(cursor: FrozenListCursor) -> DeleteMissingTargetWork {
    Ok(Ok(cursor))
}

fn drive_association_work(cursor: FrozenAssociationCursor) -> DeleteMissingTargetWork {
    Ok(Err(cursor))
}

fn finish_list_child_work(cursor: FrozenListCursor) -> DeleteMissingTargetWork {
    Err(Ok(cursor))
}

fn finish_association_child_work(
    cursor: FrozenAssociationCursor,
    entry: AssociationEntryForm,
) -> DeleteMissingTargetWork {
    Err(Err(Ok((cursor, entry))))
}

fn finish_association_predicate_work(
    cursor: FrozenAssociationCursor,
    entry: AssociationEntryForm,
) -> DeleteMissingTargetWork {
    Err(Err(Err((cursor, entry))))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::install_core;

    fn core_session() -> Session {
        let mut session = Session::new();
        install_core(&mut session);
        session
    }

    fn mixed_fixture(session: &mut Session) -> Expr {
        session
            .lower_text(
                r#"{
                    outerBefore,
                    Association[
                        rootWitness -> "root",
                        assocBranch -> Association[
                            assocBefore -> "before",
                            assocDrop -> Missing[],
                            assocDelayed :> "delayed",
                            assocAfter -> "after"
                        ],
                        listBranch -> {
                            listBefore,
                            listDrop -> Missing[],
                            Association[
                                nestedBefore -> "nested-before",
                                nestedDrop -> Missing[],
                                nestedAfter -> "nested-after"
                            ],
                            listAfter
                        },
                        rootAfter -> "root-after"
                    ],
                    outerAfter
                }"#,
            )
            .expect("mixed target fixture should lower")
    }

    fn mixed_expected(session: &mut Session) -> Expr {
        session
            .lower_text(
                r#"{
                    outerBefore,
                    Association[
                        rootWitness -> "root",
                        assocBranch -> Association[
                            assocBefore -> "before",
                            assocDelayed :> "delayed",
                            assocAfter -> "after"
                        ],
                        listBranch -> {
                            listBefore,
                            Association[
                                nestedBefore -> "nested-before",
                                nestedDrop -> Missing[],
                                nestedAfter -> "nested-after"
                            ],
                            listAfter
                        },
                        rootAfter -> "root-after"
                    ],
                    outerAfter
                }"#,
            )
            .expect("mixed target expectation should lower")
    }

    #[test]
    fn target_front_handoff_carries_live_cursors_and_completed_prefixes() {
        let mut bounded_session = core_session();
        let bounded_fixture = mixed_fixture(&mut bounded_session);
        let bounded = drive_owned_target_cursor(
            &mut bounded_session,
            OwnedTargetCursor::List(OwnedListCursor {
                items: list_items(&bounded_fixture).unwrap().to_vec(),
                level: 3,
                scan: DeleteMissingDepth::Finite(0),
                index: 0,
                kept: Vec::new(),
            }),
            0,
        );
        match bounded {
            Ok(TargetTraversal::Exhausted(continuation)) => {
                assert!(continuation.work_len() > 0, "handoff must carry live work");
            }
            Ok(TargetTraversal::Complete(_)) => panic!("front zero must exhaust"),
            Err(_) => panic!("non-suspending fixture must not fail"),
        }

        let mut front_session = core_session();
        let front_fixture = mixed_fixture(&mut front_session);
        let front_bounded = drive_owned_target_cursor(
            &mut front_session,
            OwnedTargetCursor::List(OwnedListCursor {
                items: list_items(&front_fixture).unwrap().to_vec(),
                level: 3,
                scan: DeleteMissingDepth::Finite(0),
                index: 0,
                kept: Vec::new(),
            }),
            RECURSIVE_DELETE_MISSING_TARGET_DEPTH,
        );
        assert!(matches!(front_bounded, Ok(TargetTraversal::Complete(_))));

        let mut tail_session = core_session();
        let tail_fixture = mixed_fixture(&mut tail_session);
        let through_tail = delete_missing_at_target_level_with_front_depth(
            &mut tail_session,
            &tail_fixture,
            3,
            DeleteMissingDepth::Finite(0),
            0,
        )
        .unwrap()
        .unwrap();
        let expected = mixed_expected(&mut tail_session);

        let mut complete_session = core_session();
        let complete_fixture = mixed_fixture(&mut complete_session);
        let through_front = delete_missing_at_target_level_with_front_depth(
            &mut complete_session,
            &complete_fixture,
            3,
            DeleteMissingDepth::Finite(0),
            RECURSIVE_DELETE_MISSING_TARGET_DEPTH,
        )
        .unwrap()
        .unwrap();

        assert_eq!(through_tail, expected);
        assert_eq!(through_front, expected);
    }

    #[test]
    fn resume_target_work_given_a_carried_prefix_does_not_replay_it() {
        let mut session = core_session();
        session.eval_text("seen = {}").unwrap();
        let first = session
            .lower_text(
                r#"Association[
                    firstDelayed :> (AppendTo[seen, firstRan]; firstValue),
                    firstTail -> firstTailValue
                ]"#,
            )
            .unwrap();
        let second = session
            .lower_text(
                r#"Association[
                    secondDrop -> Missing[],
                    secondKeep -> secondValue
                ]"#,
            )
            .unwrap();

        let first_result = match drive_owned_association_cursor(
            &mut session,
            OwnedAssociationCursor {
                entries: association_entry_forms_from_expr(&first).unwrap(),
                level: 1,
                scan: DeleteMissingDepth::Finite(0),
                index: 0,
                kept: Vec::new(),
            },
            RECURSIVE_DELETE_MISSING_TARGET_DEPTH,
        ) {
            Ok(TargetTraversal::Complete(value)) => value,
            Ok(TargetTraversal::Exhausted(_)) => panic!("shallow prefix must complete"),
            Err(_) => panic!("prefix callback must not suspend"),
        };
        assert_eq!(
            session.eval_text("seen").unwrap().value.to_string(),
            "List[firstRan]"
        );

        let continuation = drive_owned_list_cursor(
            &mut session,
            OwnedListCursor {
                items: vec![first, second],
                level: 2,
                scan: DeleteMissingDepth::Finite(0),
                index: 1,
                kept: vec![first_result],
            },
            0,
        );
        let continuation = match continuation {
            Ok(TargetTraversal::Exhausted(continuation)) => continuation,
            Ok(TargetTraversal::Complete(_)) => panic!("front zero must exhaust after the prefix"),
            Err(_) => panic!("handoff fixture must not suspend"),
        };
        assert!(continuation.work_len() > 0);

        let result =
            resume_delete_missing_target_work(&mut session, continuation.into_work(), None)
                .unwrap();
        let expected = session
            .lower_text(
                r#"{
                    Association[
                        firstDelayed :> (AppendTo[seen, firstRan]; firstValue),
                        firstTail -> firstTailValue
                    ],
                    Association[secondKeep -> secondValue]
                }"#,
            )
            .unwrap();
        assert_eq!(result, expected);
        assert_eq!(
            session.eval_text("seen").unwrap().value.to_string(),
            "List[firstRan]"
        );
    }
}
