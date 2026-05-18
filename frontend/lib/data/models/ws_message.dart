import 'dart:convert';

// ──────────────────────────────────────────────────────────────────────────────
// Sealed WsMessage hierarchy
// ──────────────────────────────────────────────────────────────────────────────
//
// Every inbound JSON frame from the backend maps to one of these subtypes.
// The discriminator is the `"type"` field.
//
// Example:
// ```dart
// final msg = WsMessage.fromJson(json);
// switch (msg) {
//   case MoveAppliedMessage(:final state): ...
//   case WsErrorMessage(:final code): ...
//   ...
// }
// ```

/// Base class for all WebSocket messages received from the backend.
sealed class WsMessage {
  const WsMessage();

  /// Deserialises a JSON map produced by the backend into the appropriate
  /// [WsMessage] subtype.  Unknown `"type"` values produce [UnknownMessage].
  factory WsMessage.fromJson(Map<String, dynamic> json) {
    final type = json['type'] as String? ?? '';
    return switch (type) {
      'game_created' => GameCreatedMessage.fromJson(json),
      'state_snapshot' => StateSnapshotMessage.fromJson(json),
      'move_applied' => MoveAppliedMessage.fromJson(json),
      'pass_applied' => PassAppliedMessage.fromJson(json),
      'game_finished' => GameFinishedMessage.fromJson(json),
      'game_joined' => GameJoinedMessage.fromJson(json),
      'legal_moves' => LegalMovesMessage.fromJson(json),
      'score_report' => ScoreReportMessage.fromJson(json),
      'kicked' => KickedMessage.fromJson(json),
      'error' => WsErrorMessage.fromJson(json),
      _ => UnknownMessage(type: type, raw: json),
    };
  }

  /// Convenience: parse a raw JSON string frame.
  static WsMessage fromRawString(String frame) =>
      WsMessage.fromJson(jsonDecode(frame) as Map<String, dynamic>);
}

// ──────────────────────────────────────────────────────────────────────────────
// State-carrying events (game_created / state_snapshot / move_applied /
//   pass_applied / game_finished / game_joined all share the same "state" field)
// ──────────────────────────────────────────────────────────────────────────────

/// Emitted after a game is successfully created.  Broadcast to all subscribers.
final class GameCreatedMessage extends WsMessage {
  final String gameId;
  final Map<String, dynamic> stateJson;

  const GameCreatedMessage({required this.gameId, required this.stateJson});

  factory GameCreatedMessage.fromJson(Map<String, dynamic> json) =>
      GameCreatedMessage(
        gameId: json['game_id'] as String,
        stateJson: (json['state'] as Map<String, dynamic>?) ?? {},
      );
}

/// Unicast snapshot sent to a connection on (re-)subscribe or `request_state`.
final class StateSnapshotMessage extends WsMessage {
  final String gameId;
  final Map<String, dynamic> stateJson;

  const StateSnapshotMessage({required this.gameId, required this.stateJson});

  factory StateSnapshotMessage.fromJson(Map<String, dynamic> json) =>
      StateSnapshotMessage(
        gameId: json['game_id'] as String,
        stateJson: (json['state'] as Map<String, dynamic>?) ?? {},
      );
}

/// Broadcast after a piece is successfully placed.
final class MoveAppliedMessage extends WsMessage {
  final String gameId;
  final String response;
  final Map<String, dynamic> stateJson;

  const MoveAppliedMessage({
    required this.gameId,
    required this.response,
    required this.stateJson,
  });

  factory MoveAppliedMessage.fromJson(Map<String, dynamic> json) =>
      MoveAppliedMessage(
        gameId: json['game_id'] as String,
        response: json['response'] as String? ?? '',
        stateJson: (json['state'] as Map<String, dynamic>?) ?? {},
      );
}

/// Broadcast after a player passes their turn.
final class PassAppliedMessage extends WsMessage {
  final String gameId;
  final String response;
  final Map<String, dynamic> stateJson;

  const PassAppliedMessage({
    required this.gameId,
    required this.response,
    required this.stateJson,
  });

  factory PassAppliedMessage.fromJson(Map<String, dynamic> json) =>
      PassAppliedMessage(
        gameId: json['game_id'] as String,
        response: json['response'] as String? ?? '',
        stateJson: (json['state'] as Map<String, dynamic>?) ?? {},
      );
}

/// Broadcast when all players are blocked and the game ends.
final class GameFinishedMessage extends WsMessage {
  final String gameId;
  final String response;
  final Map<String, dynamic> stateJson;

  const GameFinishedMessage({
    required this.gameId,
    required this.response,
    required this.stateJson,
  });

  factory GameFinishedMessage.fromJson(Map<String, dynamic> json) =>
      GameFinishedMessage(
        gameId: json['game_id'] as String,
        response: json['response'] as String? ?? '',
        stateJson: (json['state'] as Map<String, dynamic>?) ?? {},
      );
}

/// Emitted to the AI player's connection when it joins / is attached.
final class GameJoinedMessage extends WsMessage {
  final String gameId;
  final Map<String, dynamic> stateJson;

  const GameJoinedMessage({required this.gameId, required this.stateJson});

  factory GameJoinedMessage.fromJson(Map<String, dynamic> json) =>
      GameJoinedMessage(
        gameId: json['game_id'] as String,
        stateJson: (json['state'] as Map<String, dynamic>?) ?? {},
      );
}

// ──────────────────────────────────────────────────────────────────────────────
// Non-state events
// ──────────────────────────────────────────────────────────────────────────────

/// Unicast response to `request_legal_moves`.
final class LegalMovesMessage extends WsMessage {
  final String gameId;
  final String playerId;
  final String color;

  /// Raw move list – each entry has: piece_id, orientation_id, row, col,
  /// board_index, score_delta.
  final List<Map<String, dynamic>> moves;

  const LegalMovesMessage({
    required this.gameId,
    required this.playerId,
    required this.color,
    required this.moves,
  });

  factory LegalMovesMessage.fromJson(Map<String, dynamic> json) =>
      LegalMovesMessage(
        gameId: json['game_id'] as String,
        playerId: json['player_id'] as String,
        color: json['color'] as String,
        moves: (json['moves'] as List<dynamic>)
            .cast<Map<String, dynamic>>(),
      );
}

/// Unicast response to `request_score`.
final class ScoreReportMessage extends WsMessage {
  final String gameId;
  final String scoring;

  /// Each entry: `{ "player_id": String, "score": int }`.
  final List<Map<String, dynamic>> entries;

  const ScoreReportMessage({
    required this.gameId,
    required this.scoring,
    required this.entries,
  });

  factory ScoreReportMessage.fromJson(Map<String, dynamic> json) {
    final score = json['score'] as Map<String, dynamic>? ?? {};
    return ScoreReportMessage(
      gameId: json['game_id'] as String,
      scoring: score['scoring'] as String? ?? 'basic',
      entries: (score['entries'] as List<dynamic>? ?? [])
          .cast<Map<String, dynamic>>(),
    );
  }
}

/// Sent to the previous seat holder when another connection takes over.
final class KickedMessage extends WsMessage {
  final String reason;

  const KickedMessage({required this.reason});

  factory KickedMessage.fromJson(Map<String, dynamic> json) =>
      KickedMessage(reason: json['reason'] as String? ?? 'unknown');
}

/// Protocol-level error from the backend.
final class WsErrorMessage extends WsMessage {
  final String code;
  final String message;

  const WsErrorMessage({required this.code, required this.message});

  factory WsErrorMessage.fromJson(Map<String, dynamic> json) =>
      WsErrorMessage(
        code: json['code'] as String? ?? 'unknown',
        message: json['message'] as String? ?? '',
      );
}

/// Fallback for any unrecognised message type.
final class UnknownMessage extends WsMessage {
  final String type;
  final Map<String, dynamic> raw;

  const UnknownMessage({required this.type, required this.raw});
}
