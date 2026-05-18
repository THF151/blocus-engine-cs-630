import 'package:shared_preferences/shared_preferences.dart';

// Storage keys
const String _kServerUrl = 'blocus_server_url';
const String _kPlayerName = 'blocus_player_name';

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
          'Invalid server URL "$url": must start with ws:// or wss://');
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

  // ── Factory ─────────────────────────────────────────────────────────────────

  /// Creates a [PreferencesService] by loading the shared preferences store.
  static Future<PreferencesService> create() async {
    final prefs = await SharedPreferences.getInstance();
    return PreferencesService(prefs);
  }
}
