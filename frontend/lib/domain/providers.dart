import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:shared_preferences/shared_preferences.dart';
import '../data/game_repository.dart';
import '../data/preferences_service.dart';
import '../data/websocket_service.dart';
import 'game_notifier.dart';
import 'lobby_notifier.dart';
import 'piece_selection_notifier.dart';

// ──────────────────────────────────────────────────────────────────────────────
// Infrastructure providers (singletons)
// ──────────────────────────────────────────────────────────────────────────────

/// Provides the [SharedPreferences] instance.
///
/// Must be overridden in [main] via
/// `sharedPrefsProvider.overrideWithValue(prefs)` after awaiting
/// `SharedPreferences.getInstance()`.  The default body throws so that
/// missing overrides are caught at startup rather than silently.
final sharedPrefsProvider = Provider<SharedPreferences>((ref) {
  throw UnimplementedError(
    'sharedPrefsProvider must be overridden in main() before runApp()',
  );
});

/// Provides the [PreferencesService] backed by [sharedPrefsProvider].
final preferencesServiceProvider = Provider<PreferencesService>((ref) {
  return PreferencesService(ref.watch(sharedPrefsProvider));
});

/// Provides the singleton [WebSocketService].
///
/// The service is disposed automatically when the provider is no longer
/// referenced (provider auto-dispose is intentionally NOT set here because
/// the connection must persist across screen navigations).
final webSocketServiceProvider = Provider<WebSocketService>((ref) {
  final svc = WebSocketService();
  ref.onDispose(svc.dispose);
  return svc;
});

/// Provides the [GameRepository] wired to the active [WebSocketService].
final gameRepositoryProvider = Provider<GameRepository>((ref) {
  final ws = ref.watch(webSocketServiceProvider);
  return GameRepository(ws);
});

// ──────────────────────────────────────────────────────────────────────────────
// Domain state providers
// ──────────────────────────────────────────────────────────────────────────────

/// Provides the [LobbyNotifier] that manages game-creation / join state.
final lobbyNotifierProvider = StateNotifierProvider<LobbyNotifier, LobbyState>((
  ref,
) {
  final ws = ref.watch(webSocketServiceProvider);
  final repo = ref.watch(gameRepositoryProvider);
  return LobbyNotifier(ws, repo);
});

/// Provides the [GameNotifier] that manages in-game state.
///
/// Family parameter: the `game_id` string, so multiple simultaneous game
/// states are theoretically supported.
final gameNotifierProvider =
    StateNotifierProvider.family<GameNotifier, GameNotifierState, String>((
      ref,
      gameId,
    ) {
      final ws = ref.watch(webSocketServiceProvider);
      final repo = ref.watch(gameRepositoryProvider);
      return GameNotifier(ws, repo, gameId);
    });

/// Provides the piece-selection notifier that tracks which piece is currently
/// selected and its orientation.
final pieceSelectionProvider =
    StateNotifierProvider<PieceSelectionNotifier, PieceSelectionState>((ref) {
      return PieceSelectionNotifier();
    });

// ──────────────────────────────────────────────────────────────────────────────
// Derived / convenience providers
// ──────────────────────────────────────────────────────────────────────────────

/// Connection-state stream exposed as a [StreamProvider] for reactive UI.
final connectionStateProvider = StreamProvider<WsConnectionState>((ref) {
  final ws = ref.watch(webSocketServiceProvider);
  return ws.connectionState;
});

/// The current board cell size in logical pixels.
///
/// Updated by [GameBoard] on every layout pass so that piece widgets
/// can draw their cells at exactly the same scale as the board cells.
/// Defaults to 24 (a reasonable starting value before the board is laid out).
final boardCellSizeProvider = StateProvider<double>((ref) => 24.0);
