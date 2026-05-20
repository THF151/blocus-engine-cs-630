import 'dart:async';

import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:uuid/uuid.dart';

import '../data/game_repository.dart';
import '../data/models/saved_game.dart';
import '../data/models/ws_message.dart';
import '../data/preferences_service.dart';
import '../data/websocket_service.dart';

// ──────────────────────────────────────────────────────────────────────────────
// Enums mirroring backend strings
// ──────────────────────────────────────────────────────────────────────────────

/// UI representation of the Blokus game mode.
enum GameModeOption {
  duo, // 2 players, 14×14
  twoPlayer, // 2 players, 20×20 (each controls 2 colours)
  threePlayer,
  fourPlayer;

  String get backendValue => switch (this) {
    GameModeOption.duo => 'duo',
    GameModeOption.twoPlayer => 'two_player',
    GameModeOption.threePlayer => 'three_player',
    GameModeOption.fourPlayer => 'four_player',
  };

  String get displayName => switch (this) {
    GameModeOption.duo => 'Duo (14×14)',
    GameModeOption.twoPlayer => 'Classic 2-Player',
    GameModeOption.threePlayer => 'Classic 3-Player',
    GameModeOption.fourPlayer => 'Classic 4-Player',
  };
}

// ──────────────────────────────────────────────────────────────────────────────
// Helpers
// ──────────────────────────────────────────────────────────────────────────────

/// Returns `true` when the local human player is the active controller for
/// [currentColor].
///
/// In three-player mode the green color is shared in a fixed rotation:
/// index 0 → human (blue player), index 1 → ai1, index 2 → ai2, then wraps.
/// For every other mode or color the human is always the controller.
bool isHumanControlledTurn({
  required GameModeOption mode,
  required String currentColor,
  required int? sharedColorTurnIndex,
}) {
  if (mode != GameModeOption.threePlayer) return true;
  if (currentColor != 'green') return true;
  final idx = sharedColorTurnIndex;
  return idx == null || idx % 3 == 0;
}

// ──────────────────────────────────────────────────────────────────────────────
// State
// ──────────────────────────────────────────────────────────────────────────────

/// Phase of the lobby / game-creation flow.
enum LobbyPhase {
  idle, // Nothing started yet
  creating, // Waiting for game_created event
  joining, // Waiting for state_snapshot after subscribe_game
  ready, // Game created or joined; transition to game screen
  error,
}

/// Immutable state for the lobby / home screen.
class LobbyState {
  final LobbyPhase phase;

  // Configuration
  final GameModeOption mode;
  final String scoring; // 'basic' | 'advanced'
  final String localPlayerId;

  /// Colour → player_id mapping set during game creation.
  final Map<String, String> colorToPlayerId;

  /// Player-id → display name (e.g. 'Alice', 'AI Player 1').
  final Map<String, String> playerNames;

  /// Optional human-readable game name shown in the AppBar.
  final String gameName;

  /// The game_id (generated locally or entered by the user for join flow).
  final String gameId;

  /// Which colours the local player controls (derived from [colorToPlayerId]).
  final Set<String> localColors;

  final String? errorMessage;

  const LobbyState({
    this.phase = LobbyPhase.idle,
    this.mode = GameModeOption.fourPlayer,
    this.scoring = 'basic',
    this.localPlayerId = '',
    this.colorToPlayerId = const {},
    this.playerNames = const {},
    this.gameName = '',
    this.gameId = '',
    this.localColors = const {},
    this.errorMessage,
  });

  bool get isLoading =>
      phase == LobbyPhase.creating || phase == LobbyPhase.joining;

  LobbyState copyWith({
    LobbyPhase? phase,
    GameModeOption? mode,
    String? scoring,
    String? localPlayerId,
    Map<String, String>? colorToPlayerId,
    Map<String, String>? playerNames,
    String? gameName,
    String? gameId,
    Set<String>? localColors,
    String? errorMessage,
    bool clearError = false,
  }) => LobbyState(
    phase: phase ?? this.phase,
    mode: mode ?? this.mode,
    scoring: scoring ?? this.scoring,
    localPlayerId: localPlayerId ?? this.localPlayerId,
    colorToPlayerId: colorToPlayerId ?? this.colorToPlayerId,
    playerNames: playerNames ?? this.playerNames,
    gameName: gameName ?? this.gameName,
    gameId: gameId ?? this.gameId,
    localColors: localColors ?? this.localColors,
    errorMessage: clearError ? null : errorMessage ?? this.errorMessage,
  );
}

// ──────────────────────────────────────────────────────────────────────────────
// Notifier
// ──────────────────────────────────────────────────────────────────────────────

/// Manages game-creation and join logic prior to entering the game screen.
class LobbyNotifier extends StateNotifier<LobbyState> {
  final WebSocketService _ws;
  final GameRepository _repo;
  final PreferencesService _prefs;
  StreamSubscription<WsMessage>? _sub;

  LobbyNotifier(this._ws, this._repo, this._prefs) : super(const LobbyState()) {
    _sub = _ws.messages.listen(_handleMessage);
  }

  // ── Configuration setters ───────────────────────────────────────────────────

  void setMode(GameModeOption mode) {
    state = state.copyWith(
      mode: mode,
      // Duo is always advanced scoring
      scoring: mode == GameModeOption.duo ? 'advanced' : state.scoring,
    );
  }

  void setScoring(String scoring) => state = state.copyWith(scoring: scoring);

  void setLocalPlayerId(String id) => state = state.copyWith(localPlayerId: id);

  void setPlayerNames(Map<String, String> names) =>
      state = state.copyWith(playerNames: names);

  void setGameName(String name) => state = state.copyWith(gameName: name);

  // ── Actions ─────────────────────────────────────────────────────────────────

  /// Creates a 4-player game assigning [colorToPlayerId] slots.
  ///
  /// The caller is responsible for building [colorToPlayerId] from the form
  /// fields.  [localPlayerId] must already be set.
  void createGame({
    required String gameId,
    required Map<String, String> colorToPlayerId,
    String? firstColor,
  }) {
    final mode = state.mode;
    final localId = state.localPlayerId;

    final localColors =
        colorToPlayerId.entries
            .where((e) => e.value == localId)
            .map((e) => e.key)
            .toSet();

    state = state.copyWith(
      phase: LobbyPhase.creating,
      gameId: gameId,
      colorToPlayerId: colorToPlayerId,
      localColors: localColors,
      clearError: true,
    );

    switch (mode) {
      case GameModeOption.duo:
        _repo.createDuoGame(
          gameId: gameId,
          blackPlayerId: colorToPlayerId['black'] ?? '',
          whitePlayerId: colorToPlayerId['white'] ?? '',
          firstColor: firstColor ?? 'black',
        );
      case GameModeOption.twoPlayer:
        _repo.createTwoPlayerGame(
          gameId: gameId,
          blueRedPlayerId: colorToPlayerId['blue_red'] ?? localId,
          yellowGreenPlayerId: colorToPlayerId['yellow_green'] ?? '',
          scoring: state.scoring,
        );
      case GameModeOption.threePlayer:
        // The engine requires exactly 3 players in the shared-green rotation
        // cycle (one per owned color). Green is played in turns: blue → yellow
        // → red → blue → …
        _repo.createThreePlayerGame(
          gameId: gameId,
          bluePlayerId: colorToPlayerId['blue'] ?? '',
          yellowPlayerId: colorToPlayerId['yellow'] ?? '',
          redPlayerId: colorToPlayerId['red'] ?? '',
          sharedGreenPlayers: [
            colorToPlayerId['blue'] ?? '',
            colorToPlayerId['yellow'] ?? '',
            colorToPlayerId['red'] ?? '',
          ],
          scoring: state.scoring,
        );
      case GameModeOption.fourPlayer:
        _repo.createFourPlayerGame(
          gameId: gameId,
          bluePlayerId: colorToPlayerId['blue'] ?? '',
          yellowPlayerId: colorToPlayerId['yellow'] ?? '',
          redPlayerId: colorToPlayerId['red'] ?? '',
          greenPlayerId: colorToPlayerId['green'] ?? '',
          firstColor: firstColor ?? 'blue',
          scoring: state.scoring,
        );
    }
  }

  /// Joins an existing game by subscribing with the local player's ID.
  void joinGame(String gameId) {
    if (state.localPlayerId.isEmpty) {
      state = state.copyWith(errorMessage: 'Player name must not be empty.');
      return;
    }
    state = state.copyWith(
      phase: LobbyPhase.joining,
      gameId: gameId,
      clearError: true,
    );
    _repo.joinGame(gameId, state.localPlayerId);
  }

  /// Connects the WebSocket to [serverUrl] and updates connection status.
  Future<void> connectToServer(String serverUrl) async {
    await _ws.connect(serverUrl);
  }

  /// Creates a game entry in local persistence without calling the backend.
  ///
  /// [mode] and [scoring] are read from the current notifier state so the
  /// caller only needs to set them via [setMode] / [setScoring] beforehand.
  /// Returns the newly generated game ID.
  String createLocalGame({
    required String creatorName,
    required String gameName,
  }) {
    const uuid = Uuid();
    final gameId = uuid.v4();
    final mode = state.mode;
    final scoring = mode == GameModeOption.duo ? 'advanced' : state.scoring;

    final slots = _slotsForMode(mode);
    final players = {
      for (final s in slots) s: '',
      slots[0]: creatorName.trim(), // creator in first slot
    };

    final saved = SavedGame(
      gameId: gameId,
      gameName: gameName.trim(),
      mode: mode.backendValue,
      scoring: scoring,
      players: players,
      createdAt: DateTime.now(),
    );
    _prefs.upsertGame(saved);
    return gameId;
  }

  /// Connects to [serverUrl], sets up state from [savedGame] and
  /// [finalSlotNames], then triggers the actual backend game-creation call.
  ///
  /// [finalSlotNames] maps each colour slot to the display name that should
  /// occupy it (empty string → AI player).
  Future<void> startGame({
    required SavedGame savedGame,
    required String localPlayerName,
    required Map<String, String> finalSlotNames,
    required String serverUrl,
  }) async {
    const uuid = Uuid();
    final Map<String, String> colorToPlayerId = {};
    final Map<String, String> playerNames = {};
    String? localId;
    int aiCount = 0;

    for (final entry in finalSlotNames.entries) {
      final slot = entry.key;
      final name = entry.value.trim();
      if (name.isNotEmpty && name == localPlayerName && localId == null) {
        localId = uuid.v4();
        colorToPlayerId[slot] = localId;
        playerNames[localId] = name;
      } else if (name.isNotEmpty) {
        final pid = uuid.v4();
        colorToPlayerId[slot] = pid;
        playerNames[pid] = name;
      } else {
        aiCount++;
        final aiId = uuid.v4();
        colorToPlayerId[slot] = aiId;
        playerNames[aiId] = 'AI Player $aiCount';
      }
    }

    localId ??= uuid.v4(); // fallback: local player not found in any slot

    // Three-player: shared green colour is controlled by the local player.
    if (savedGame.mode == 'three_player') {
      colorToPlayerId['green'] = localId;
    }

    final modeEnum = GameModeOption.values.firstWhere(
      (m) => m.backendValue == savedGame.mode,
      orElse: () => GameModeOption.fourPlayer,
    );

    setMode(modeEnum);
    setScoring(savedGame.scoring);
    setGameName(savedGame.gameName);
    setLocalPlayerId(localId);
    setPlayerNames(playerNames);

    await connectToServer(serverUrl);

    final firstColor = switch (savedGame.mode) {
      'duo' => 'black',
      'four_player' => 'blue',
      _ => null,
    };

    createGame(
      gameId: savedGame.gameId,
      colorToPlayerId: colorToPlayerId,
      firstColor: firstColor,
    );
  }

  /// Resets the lobby to its initial state (e.g. after returning to home).
  void reset() => state = const LobbyState();

  // ── WS event handlers ───────────────────────────────────────────────────────

  void _handleMessage(WsMessage msg) {
    switch (msg) {
      case GameCreatedMessage(:final gameId) when gameId == state.gameId:
        state = state.copyWith(phase: LobbyPhase.ready);

      case StateSnapshotMessage(:final gameId) when gameId == state.gameId:
        if (state.phase == LobbyPhase.joining) {
          state = state.copyWith(phase: LobbyPhase.ready);
        }

      case WsErrorMessage(:final code, :final message):
        if (state.isLoading) {
          state = state.copyWith(
            phase: LobbyPhase.error,
            errorMessage: '[$code] $message',
          );
        }

      default:
        break;
    }
  }

  // ── Helpers ──────────────────────────────────────────────────────────────────

  List<String> _slotsForMode(GameModeOption mode) => switch (mode) {
    GameModeOption.duo => ['black', 'white'],
    GameModeOption.twoPlayer => ['blue_red', 'yellow_green'],
    GameModeOption.threePlayer => ['blue', 'yellow', 'red'],
    GameModeOption.fourPlayer => ['blue', 'yellow', 'red', 'green'],
  };

  @override
  void dispose() {
    _sub?.cancel();
    super.dispose();
  }
}
