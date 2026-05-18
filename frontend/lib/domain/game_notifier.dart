import 'dart:async';

import 'package:flutter/foundation.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../data/game_repository.dart';
import '../data/models/game_state_model.dart';
import '../data/models/legal_move_model.dart';
import '../data/models/ws_message.dart';
import '../data/websocket_service.dart';

// ──────────────────────────────────────────────────────────────────────────────
// State
// ──────────────────────────────────────────────────────────────────────────────

/// Immutable snapshot of everything the game screen needs.
class GameNotifierState {
  /// Current state delivered by the backend (null until first snapshot).
  final GameStateModel? gameState;

  /// Legal moves for the current local player's colour.  Populated whenever
  /// it is the local player's turn and cleared otherwise.
  final List<LegalMoveModel> legalMoves;

  /// Set of piece IDs that the local player has already placed.
  /// Used to grey out spent pieces in the tray.
  final Set<int> usedPieceIds;

  /// Final score report (populated once the game is finished).
  final ScoreReportModel? scoreReport;

  /// True while waiting for a response to a place/pass command.
  final bool isLoadingMove;

  /// Non-null when the backend returns an error code.
  final String? errorMessage;

  const GameNotifierState({
    this.gameState,
    this.legalMoves = const [],
    this.usedPieceIds = const {},
    this.scoreReport,
    this.isLoadingMove = false,
    this.errorMessage,
  });

  /// Convenience: current board matrix for rendering.
  List<List<String?>> get boardMatrix =>
      gameState?.toMatrix() ?? List.generate(20, (_) => List.filled(20, null));

  GameNotifierState copyWith({
    GameStateModel? gameState,
    List<LegalMoveModel>? legalMoves,
    Set<int>? usedPieceIds,
    ScoreReportModel? scoreReport,
    bool? isLoadingMove,
    String? errorMessage,
    bool clearError = false,
  }) => GameNotifierState(
    gameState: gameState ?? this.gameState,
    legalMoves: legalMoves ?? this.legalMoves,
    usedPieceIds: usedPieceIds ?? this.usedPieceIds,
    scoreReport: scoreReport ?? this.scoreReport,
    isLoadingMove: isLoadingMove ?? this.isLoadingMove,
    errorMessage: clearError ? null : errorMessage ?? this.errorMessage,
  );
}

// ──────────────────────────────────────────────────────────────────────────────
// Notifier
// ──────────────────────────────────────────────────────────────────────────────

/// Manages all in-game state for a single game identified by [gameId].
///
/// On creation the notifier subscribes to the WS stream and dispatches
/// incoming events to dedicated handlers.  It also requests the initial
/// state snapshot and – when it is the local player's turn – fetches
/// legal moves automatically.
class GameNotifier extends StateNotifier<GameNotifierState> {
  final WebSocketService _ws;
  final GameRepository _repo;
  final String gameId;

  /// The player_id of the local user (set via [setLocalIdentity]).
  String? _localPlayerId;

  /// The set of colour strings the local player controls.
  Set<String> _localColors = {};

  StreamSubscription<WsMessage>? _sub;

  GameNotifier(this._ws, this._repo, this.gameId)
    : super(const GameNotifierState()) {
    _sub = _ws.messages.listen(_handleMessage);
  }

  // ── Public API ──────────────────────────────────────────────────────────────

  /// Sets the local player's identity and subscribes / claims the seat.
  ///
  /// Call this once after the notifier is created (from the lobby or home
  /// screen).  [localColors] contains the colour strings this player controls
  /// (1 colour for 4-player, 2 for 2-player, etc.).
  void setLocalIdentity({
    required String playerId,
    required Set<String> localColors,
  }) {
    _localPlayerId = playerId;
    _localColors = localColors;
    _repo.subscribeGame(gameId, playerId: playerId);
  }

  /// Subscribes as a spectator (no moves possible).
  void subscribeAsSpectator() {
    _repo.subscribeGame(gameId);
  }

  /// Places [pieceId] in [orientationId] at ([row], [col]).
  ///
  /// The color is resolved from [_localColors] by matching [currentColor].
  void placeMove(LegalMoveModel move) {
    if (_localPlayerId == null) return;
    final color = state.gameState?.currentColor;
    if (color == null || !_localColors.contains(color)) return;

    final newUsedIds = {...state.usedPieceIds, move.pieceId};
    state = state.copyWith(
      isLoadingMove: true,
      usedPieceIds: newUsedIds,
      clearError: true,
    );
    _repo.placeMove(
      gameId: gameId,
      playerId: _localPlayerId!,
      color: color,
      pieceId: move.pieceId,
      orientationId: move.orientationId,
      row: move.row,
      col: move.col,
    );
  }

  /// Passes the current turn.
  void passMove() {
    if (_localPlayerId == null) return;
    final color = state.gameState?.currentColor;
    if (color == null || !_localColors.contains(color)) return;

    state = state.copyWith(isLoadingMove: true, clearError: true);
    _repo.passMove(gameId: gameId, playerId: _localPlayerId!, color: color);
  }

  /// Requests legal moves for the current player colour.
  void refreshLegalMoves() {
    if (_localPlayerId == null) return;
    final color = state.gameState?.currentColor;
    if (color == null || !_localColors.contains(color)) {
      // Not our turn – clear any stale legal moves.
      state = state.copyWith(legalMoves: []);
      return;
    }
    _repo.requestLegalMoves(gameId, _localPlayerId!, color);
  }

  /// Requests the final score report.
  void requestScore() => _repo.requestScore(gameId);

  /// Attaches an AI for the given colour seat.
  void attachAi(String playerId, String color) =>
      _repo.attachAi(gameId: gameId, playerId: playerId, color: color);

  // ── WS event handlers ───────────────────────────────────────────────────────

  void _handleMessage(WsMessage msg) {
    switch (msg) {
      case GameCreatedMessage(:final gameId, :final stateJson)
          when gameId == this.gameId:
        _applyStateJson(stateJson);

      case StateSnapshotMessage(:final gameId, :final stateJson)
          when gameId == this.gameId:
        _applyStateJson(stateJson);

      case MoveAppliedMessage(:final gameId, :final stateJson)
          when gameId == this.gameId:
        _applyStateJson(stateJson);
        _onMoveApplied(stateJson);

      case PassAppliedMessage(:final gameId, :final stateJson)
          when gameId == this.gameId:
        _applyStateJson(stateJson);

      case GameFinishedMessage(:final gameId, :final stateJson)
          when gameId == this.gameId:
        _applyStateJson(stateJson);
        _repo.requestScore(gameId);

      case LegalMovesMessage(:final gameId, :final color, :final moves)
          when gameId == this.gameId && _localColors.contains(color):
        final parsed = moves
            .map((m) => LegalMoveModel.fromJson(m))
            .toList(growable: false);
        state = state.copyWith(legalMoves: parsed);

      case ScoreReportMessage(:final gameId, :final scoring, :final entries)
          when gameId == this.gameId:
        final report = ScoreReportModel.fromJson({
          'scoring': scoring,
          'entries': entries,
        });
        state = state.copyWith(scoreReport: report);

      case WsErrorMessage(:final code, :final message):
        state = state.copyWith(
          isLoadingMove: false,
          errorMessage: _friendlyError(code, message),
        );

      case KickedMessage():
        // Connection was taken over by a reconnect; handled at app level.
        break;

      case LegalMovesMessage(:final gameId, :final color)
          when gameId == this.gameId && !_localColors.contains(color):
        break;

      default:
        break;
    }
  }

  void _applyStateJson(Map<String, dynamic> json) {
    if (json.isEmpty) return;
    try {
      final newState = GameStateModel.fromJson(json);
      state = state.copyWith(
        gameState: newState,
        legalMoves: const [],
        isLoadingMove: false,
        clearError: true,
      );
      // Refresh legal moves whenever the game state updates.
      refreshLegalMoves();
    } catch (e) {
      debugPrint('[GameNotifier] Failed to parse state: $e');
    }
  }

  void _onMoveApplied(Map<String, dynamic> stateJson) {
    // Track used pieces for the local player so the tray greys them out.
    // We infer this from legal-moves responses (pieces absent from legal moves
    // have been played).  No direct tracking needed here.
  }

  /// Maps backend error codes/messages to user-facing German strings.
  String _friendlyError(String code, String message) {
    if (code == 'rule_violation' && message.contains('legal move')) {
      return 'Es sind noch Züge möglich';
    }
    return '[$code] $message';
  }

  @override
  void dispose() {
    _sub?.cancel();
    super.dispose();
  }
}
