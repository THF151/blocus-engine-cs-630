use blocus_core::{DomainError, EngineError, InputError, RuleViolation};
use std::collections::HashSet;

const RULE_CATEGORY: &str = "rule_violation";
const INPUT_CATEGORY: &str = "input_error";
const ENGINE_CATEGORY: &str = "engine_error";

const RULE_CASES: &[(RuleViolation, &str, &str)] = &[
    (
        RuleViolation::WrongPlayerTurn,
        "WrongPlayerTurn",
        "it is not this color's turn",
    ),
    (
        RuleViolation::PlayerDoesNotControlColor,
        "PlayerDoesNotControlColor",
        "player does not control this color",
    ),
    (
        RuleViolation::PieceAlreadyUsed,
        "PieceAlreadyUsed",
        "piece has already been used",
    ),
    (
        RuleViolation::OutOfBounds,
        "OutOfBounds",
        "placement is outside the playable board",
    ),
    (
        RuleViolation::Overlap,
        "Overlap",
        "placement overlaps an occupied cell",
    ),
    (
        RuleViolation::MissingCornerContact,
        "MissingCornerContact",
        "placement must touch same-color piece by corner",
    ),
    (
        RuleViolation::IllegalEdgeContact,
        "IllegalEdgeContact",
        "placement must not touch same-color piece by edge",
    ),
    (
        RuleViolation::GameAlreadyFinished,
        "GameAlreadyFinished",
        "the game has already finished",
    ),
    (
        RuleViolation::PassNotAllowedBecauseMoveExists,
        "PassNotAllowedBecauseMoveExists",
        "pass is not allowed while a legal move exists",
    ),
    (
        RuleViolation::GameNotFinished,
        "GameNotFinished",
        "the game is not finished",
    ),
];

const INPUT_CASES: &[(InputError, &str, &str)] = &[
    (
        InputError::GameIdMismatch,
        "GameIdMismatch",
        "command game ID does not match state game ID",
    ),
    (InputError::UnknownPlayer, "UnknownPlayer", "unknown player"),
    (InputError::UnknownPiece, "UnknownPiece", "unknown piece"),
    (
        InputError::UnknownOrientation,
        "UnknownOrientation",
        "unknown orientation",
    ),
    (
        InputError::InvalidBoardIndex,
        "InvalidBoardIndex",
        "invalid board index",
    ),
    (
        InputError::InvalidGameConfig,
        "InvalidGameConfig",
        "invalid game configuration",
    ),
    (
        InputError::InvalidStateVersion,
        "InvalidStateVersion",
        "invalid state version",
    ),
];

const ENGINE_CASES: &[(EngineError, &str, &str)] = &[
    (
        EngineError::CorruptedState,
        "CorruptedState",
        "corrupted game state",
    ),
    (
        EngineError::InvariantViolation,
        "InvariantViolation",
        "engine invariant violation",
    ),
    (
        EngineError::RepositoryInitializationFailed,
        "RepositoryInitializationFailed",
        "piece repository initialization failed",
    ),
];

const CONST_RULE_CODE: &str = RuleViolation::Overlap.code();
const CONST_RULE_MESSAGE: &str = RuleViolation::Overlap.message();
const CONST_RULE_CATEGORY: &str = RuleViolation::Overlap.category();

const CONST_INPUT_CODE: &str = InputError::UnknownPiece.code();
const CONST_INPUT_MESSAGE: &str = InputError::UnknownPiece.message();
const CONST_INPUT_CATEGORY: &str = InputError::UnknownPiece.category();

const CONST_ENGINE_CODE: &str = EngineError::InvariantViolation.code();
const CONST_ENGINE_MESSAGE: &str = EngineError::InvariantViolation.message();
const CONST_ENGINE_CATEGORY: &str = EngineError::InvariantViolation.category();

const CONST_DOMAIN_RULE_CODE: &str = DomainError::RuleViolation(RuleViolation::Overlap).code();
const CONST_DOMAIN_RULE_MESSAGE: &str =
    DomainError::RuleViolation(RuleViolation::Overlap).message();
const CONST_DOMAIN_RULE_CATEGORY: &str =
    DomainError::RuleViolation(RuleViolation::Overlap).category();

const CONST_DOMAIN_INPUT_CODE: &str = DomainError::InputError(InputError::UnknownPiece).code();
const CONST_DOMAIN_INPUT_MESSAGE: &str =
    DomainError::InputError(InputError::UnknownPiece).message();
const CONST_DOMAIN_INPUT_CATEGORY: &str =
    DomainError::InputError(InputError::UnknownPiece).category();

const CONST_DOMAIN_ENGINE_CODE: &str =
    DomainError::EngineError(EngineError::InvariantViolation).code();
const CONST_DOMAIN_ENGINE_MESSAGE: &str =
    DomainError::EngineError(EngineError::InvariantViolation).message();
const CONST_DOMAIN_ENGINE_CATEGORY: &str =
    DomainError::EngineError(EngineError::InvariantViolation).category();

fn assert_error_contract(
    code: &str,
    message: &str,
    expected_category: &str,
    actual_category: &str,
    error: &dyn std::fmt::Display,
) {
    assert_eq!(error.to_string(), format!("{code}: {message}"));
    assert_eq!(actual_category, expected_category);
    assert!(!code.is_empty());
    assert!(!message.is_empty());
    assert!(!actual_category.is_empty());
}

#[test]
fn rule_violation_codes_messages_categories_and_display_are_stable() {
    assert_eq!(RuleViolation::CATEGORY, RULE_CATEGORY);

    for (error, code, message) in RULE_CASES {
        assert_eq!(error.code(), *code);
        assert_eq!(error.message(), *message);
        assert_eq!(error.category(), RULE_CATEGORY);

        assert_error_contract(code, message, RULE_CATEGORY, error.category(), error);

        let domain_error = DomainError::from(*error);

        assert_eq!(domain_error.code(), *code);
        assert_eq!(domain_error.message(), *message);
        assert_eq!(domain_error.category(), RULE_CATEGORY);

        assert_error_contract(
            code,
            message,
            RULE_CATEGORY,
            domain_error.category(),
            &domain_error,
        );

        assert_eq!(domain_error, DomainError::RuleViolation(*error));
    }
}

#[test]
fn input_error_codes_messages_categories_and_display_are_stable() {
    assert_eq!(InputError::CATEGORY, INPUT_CATEGORY);

    for (error, code, message) in INPUT_CASES {
        assert_eq!(error.code(), *code);
        assert_eq!(error.message(), *message);
        assert_eq!(error.category(), INPUT_CATEGORY);

        assert_error_contract(code, message, INPUT_CATEGORY, error.category(), error);

        let domain_error = DomainError::from(*error);

        assert_eq!(domain_error.code(), *code);
        assert_eq!(domain_error.message(), *message);
        assert_eq!(domain_error.category(), INPUT_CATEGORY);

        assert_error_contract(
            code,
            message,
            INPUT_CATEGORY,
            domain_error.category(),
            &domain_error,
        );

        assert_eq!(domain_error, DomainError::InputError(*error));
    }
}

#[test]
fn engine_error_codes_messages_categories_and_display_are_stable() {
    assert_eq!(EngineError::CATEGORY, ENGINE_CATEGORY);

    for (error, code, message) in ENGINE_CASES {
        assert_eq!(error.code(), *code);
        assert_eq!(error.message(), *message);
        assert_eq!(error.category(), ENGINE_CATEGORY);

        assert_error_contract(code, message, ENGINE_CATEGORY, error.category(), error);

        let domain_error = DomainError::from(*error);

        assert_eq!(domain_error.code(), *code);
        assert_eq!(domain_error.message(), *message);
        assert_eq!(domain_error.category(), ENGINE_CATEGORY);

        assert_error_contract(
            code,
            message,
            ENGINE_CATEGORY,
            domain_error.category(),
            &domain_error,
        );

        assert_eq!(domain_error, DomainError::EngineError(*error));
    }
}

#[test]
fn all_error_types_implement_std_error() {
    fn assert_std_error<T: std::error::Error>() {}

    assert_std_error::<DomainError>();
    assert_std_error::<RuleViolation>();
    assert_std_error::<InputError>();
    assert_std_error::<EngineError>();
}

#[test]
fn domain_error_source_returns_wrapped_rule_violation() {
    let error = DomainError::from(RuleViolation::Overlap);

    let Some(source) = std::error::Error::source(&error) else {
        panic!("rule violation domain error should expose a source");
    };

    assert_eq!(
        source.downcast_ref::<RuleViolation>(),
        Some(&RuleViolation::Overlap)
    );
    assert_eq!(source.to_string(), RuleViolation::Overlap.to_string());
}

#[test]
fn domain_error_source_returns_wrapped_input_error() {
    let error = DomainError::from(InputError::UnknownPiece);

    let Some(source) = std::error::Error::source(&error) else {
        panic!("input domain error should expose a source");
    };

    assert_eq!(
        source.downcast_ref::<InputError>(),
        Some(&InputError::UnknownPiece)
    );
    assert_eq!(source.to_string(), InputError::UnknownPiece.to_string());
}

#[test]
fn domain_error_source_returns_wrapped_engine_error() {
    let error = DomainError::from(EngineError::InvariantViolation);

    let Some(source) = std::error::Error::source(&error) else {
        panic!("engine domain error should expose a source");
    };

    assert_eq!(
        source.downcast_ref::<EngineError>(),
        Some(&EngineError::InvariantViolation)
    );
    assert_eq!(
        source.to_string(),
        EngineError::InvariantViolation.to_string()
    );
}

#[test]
fn error_codes_are_unique_within_each_category() {
    let rule_codes = RULE_CASES
        .iter()
        .map(|(_, code, _)| *code)
        .collect::<HashSet<_>>();
    assert_eq!(rule_codes.len(), RULE_CASES.len());

    let input_codes = INPUT_CASES
        .iter()
        .map(|(_, code, _)| *code)
        .collect::<HashSet<_>>();
    assert_eq!(input_codes.len(), INPUT_CASES.len());

    let engine_codes = ENGINE_CASES
        .iter()
        .map(|(_, code, _)| *code)
        .collect::<HashSet<_>>();
    assert_eq!(engine_codes.len(), ENGINE_CASES.len());
}

#[test]
fn error_codes_are_globally_unique() {
    let codes = RULE_CASES
        .iter()
        .map(|(_, code, _)| *code)
        .chain(INPUT_CASES.iter().map(|(_, code, _)| *code))
        .chain(ENGINE_CASES.iter().map(|(_, code, _)| *code))
        .collect::<HashSet<_>>();

    assert_eq!(
        codes.len(),
        RULE_CASES.len() + INPUT_CASES.len() + ENGINE_CASES.len()
    );
}

#[test]
fn error_messages_are_non_empty() {
    for (_, _, message) in RULE_CASES {
        assert!(!message.is_empty());
    }

    for (_, _, message) in INPUT_CASES {
        assert!(!message.is_empty());
    }

    for (_, _, message) in ENGINE_CASES {
        assert!(!message.is_empty());
    }
}

#[test]
fn rule_violation_is_copy_ordered_and_hashable() {
    let first = RuleViolation::WrongPlayerTurn;
    let duplicate = RuleViolation::WrongPlayerTurn;
    let second = RuleViolation::Overlap;
    let copied = first;

    assert_eq!(first, copied);
    assert_eq!(first, duplicate);
    assert_ne!(first, second);
    assert!(first < second);

    let mut errors = HashSet::new();
    errors.insert(first);
    errors.insert(duplicate);
    errors.insert(second);

    assert_eq!(errors.len(), 2);
    assert!(errors.contains(&RuleViolation::WrongPlayerTurn));
    assert!(errors.contains(&RuleViolation::Overlap));
}

#[test]
fn input_error_is_copy_ordered_and_hashable() {
    let first = InputError::GameIdMismatch;
    let duplicate = InputError::GameIdMismatch;
    let second = InputError::UnknownPiece;
    let copied = first;

    assert_eq!(first, copied);
    assert_eq!(first, duplicate);
    assert_ne!(first, second);
    assert!(first < second);

    let mut errors = HashSet::new();
    errors.insert(first);
    errors.insert(duplicate);
    errors.insert(second);

    assert_eq!(errors.len(), 2);
    assert!(errors.contains(&InputError::GameIdMismatch));
    assert!(errors.contains(&InputError::UnknownPiece));
}

#[test]
fn engine_error_is_copy_ordered_and_hashable() {
    let first = EngineError::CorruptedState;
    let duplicate = EngineError::CorruptedState;
    let second = EngineError::InvariantViolation;
    let copied = first;

    assert_eq!(first, copied);
    assert_eq!(first, duplicate);
    assert_ne!(first, second);
    assert!(first < second);

    let mut errors = HashSet::new();
    errors.insert(first);
    errors.insert(duplicate);
    errors.insert(second);

    assert_eq!(errors.len(), 2);
    assert!(errors.contains(&EngineError::CorruptedState));
    assert!(errors.contains(&EngineError::InvariantViolation));
}

#[test]
fn domain_error_is_clone_debug_eq_and_send_sync() {
    fn assert_send_sync<T: Send + Sync>() {}

    assert_send_sync::<DomainError>();

    let error = DomainError::from(InputError::UnknownPiece);
    let cloned = error.clone();

    assert_eq!(error, cloned);
    assert!(format!("{error:?}").contains("UnknownPiece"));
}

#[test]
fn const_context_accessors_work_for_rule_violation() {
    assert_eq!(CONST_RULE_CODE, "Overlap");
    assert_eq!(CONST_RULE_MESSAGE, "placement overlaps an occupied cell");
    assert_eq!(CONST_RULE_CATEGORY, RULE_CATEGORY);
}

#[test]
fn const_context_accessors_work_for_input_error() {
    assert_eq!(CONST_INPUT_CODE, "UnknownPiece");
    assert_eq!(CONST_INPUT_MESSAGE, "unknown piece");
    assert_eq!(CONST_INPUT_CATEGORY, INPUT_CATEGORY);
}

#[test]
fn const_context_accessors_work_for_engine_error() {
    assert_eq!(CONST_ENGINE_CODE, "InvariantViolation");
    assert_eq!(CONST_ENGINE_MESSAGE, "engine invariant violation");
    assert_eq!(CONST_ENGINE_CATEGORY, ENGINE_CATEGORY);
}

#[test]
fn const_context_accessors_work_for_domain_error_rule_violation() {
    assert_eq!(CONST_DOMAIN_RULE_CODE, "Overlap");
    assert_eq!(
        CONST_DOMAIN_RULE_MESSAGE,
        "placement overlaps an occupied cell"
    );
    assert_eq!(CONST_DOMAIN_RULE_CATEGORY, RULE_CATEGORY);
}

#[test]
fn const_context_accessors_work_for_domain_error_input_error() {
    assert_eq!(CONST_DOMAIN_INPUT_CODE, "UnknownPiece");
    assert_eq!(CONST_DOMAIN_INPUT_MESSAGE, "unknown piece");
    assert_eq!(CONST_DOMAIN_INPUT_CATEGORY, INPUT_CATEGORY);
}

#[test]
fn const_context_accessors_work_for_domain_error_engine_error() {
    assert_eq!(CONST_DOMAIN_ENGINE_CODE, "InvariantViolation");
    assert_eq!(CONST_DOMAIN_ENGINE_MESSAGE, "engine invariant violation");
    assert_eq!(CONST_DOMAIN_ENGINE_CATEGORY, ENGINE_CATEGORY);
}
