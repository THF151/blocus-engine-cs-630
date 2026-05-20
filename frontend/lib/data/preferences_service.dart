import 'dart:convert';

import 'package:shared_preferences/shared_preferences.dart';

import 'models/saved_game.dart';

// Storage keys
const String _kServerUrl = 'blocus_server_url';
const String _kPlayerName = 'blocus_player_name';
const String _kSavedGames = 'blocus_saved_games';

/// Thin wrapper around [SharedPreferences] for persisting user settings.
///
/// Only non-sensitive preferences are stored here.  No credentials or
/// authentication tokens should ever be written via this class.
class PreferencesService {
  PreferencesService(this._prefs);

  final SharedPreferences _prefs;

  // ── Server URL ──────────────────────────────────────────────────────────────

  /// Returns the last-used server WebSocket URL, defaulting to localhost.
  String get serverUrl =>
      _prefs.getString(_kServerUrl) ?? 'ws://localhost:8000/ws';

  /// Persists [url] as the server URL after validating the scheme.
  ///
  /// Throws [ArgumentError] when the scheme is neither `ws://` nor `wss://`.
  Future<void> setServerUrl(String url) async {
    final uri = Uri.tryParse(url);
    if (uri == null || (uri.scheme != 'ws' && uri.scheme != 'wss')) {
      throw ArgumentError(
        'Invalid server URL "$url": must start with ws:// or wss://',
      );
    }
    await _prefs.setString(_kServerUrl, url);
  }

  // ── Player name ─────────────────────────────────────────────────────────────

  /// Returns the persisted player name, defaulting to `"Player"`.
  String get playerName => _prefs.getString(_kPlayerName) ?? 'Player';

  /// Persists [name].  Trims whitespace and rejects empty strings.
  Future<void> setPlayerName(String name) async {
    final trimmed = name.trim();
    if (trimmed.isEmpty) throw ArgumentError('Player name must not be empty.');
    await _prefs.setString(_kPlayerName, trimmed);
  }

  // ── Saved games ─────────────────────────────────────────────────────────────

  /// Returns all locally persisted game entries, newest first.
  List<SavedGame> get savedGames {
    final raw = _prefs.getString(_kSavedGames);
    if (raw == null) return [];
    try {
      final list = jsonDecode(raw) as List;
      return list
          .map((e) => SavedGame.fromJson(e as Map<String, dynamic>))
          .toList();
    } catch (_) {
      return [];
    }
  }

  /// Inserts or updates [game] in the persisted list.
  Future<void> upsertGame(SavedGame game) async {
    final games = savedGames;
    final idx = games.indexWhere((g) => g.gameId == game.gameId);
    if (idx >= 0) {
      games[idx] = game;
    } else {
      games.insert(0, game); // newest first
    }
    await _prefs.setString(
      _kSavedGames,
      jsonEncode(games.map((g) => g.toJson()).toList()),
    );
  }

  /// Removes the game with [gameId] from the persisted list.
  Future<void> deleteGame(String gameId) async {
    final games = savedGames.where((g) => g.gameId != gameId).toList();
    await _prefs.setString(
      _kSavedGames,
      jsonEncode(games.map((g) => g.toJson()).toList()),
    );
  }

  // ── Factory ─────────────────────────────────────────────────────────────────

  /// Creates a [PreferencesService] by loading the shared preferences store.
  static Future<PreferencesService> create() async {
    final prefs = await SharedPreferences.getInstance();
    return PreferencesService(prefs);
  }
}
