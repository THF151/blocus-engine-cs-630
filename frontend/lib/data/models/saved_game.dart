/// A game entry created locally and persisted on-device.
///
/// Created when the user clicks "Create Game" on the home screen.
/// Stored in [PreferencesService] and used to populate the "Join Game"
/// dropdown.  Deleted automatically when the score screen is shown.
class SavedGame {
  const SavedGame({
    required this.gameId,
    required this.gameName,
    required this.mode,
    required this.scoring,
    required this.players,
    required this.createdAt,
  });

  final String gameId;
  final String gameName;

  /// Backend mode string: `'duo'`, `'two_player'`, `'three_player'`,
  /// `'four_player'`.
  final String mode;

  /// `'basic'` or `'advanced'`.
  final String scoring;

  /// Colour-slot → display name.  An empty string means the slot will be
  /// filled by an AI player when the game is started.
  final Map<String, String> players;

  final DateTime createdAt;

  // ── Derived helpers ─────────────────────────────────────────────────────────

  /// Label shown in the "Select Game" dropdown.
  String get displayLabel {
    final label = gameName.isNotEmpty ? gameName : gameId.substring(0, 8);
    return '$label · $modeDisplayName';
  }

  String get modeDisplayName => switch (mode) {
    'duo' => 'Duo (14×14)',
    'two_player' => 'Classic 2-Player',
    'three_player' => 'Classic 3-Player',
    'four_player' => 'Classic 4-Player',
    _ => mode,
  };

  /// Ordered colour slots for this mode.
  List<String> get slots => switch (mode) {
    'duo' => ['black', 'white'],
    'two_player' => ['blue_red', 'yellow_green'],
    'three_player' => ['blue', 'yellow', 'red'],
    _ => ['blue', 'yellow', 'red', 'green'], // four_player default
  };

  // ── Serialisation ───────────────────────────────────────────────────────────

  Map<String, dynamic> toJson() => {
    'gameId': gameId,
    'gameName': gameName,
    'mode': mode,
    'scoring': scoring,
    'players': players,
    'createdAt': createdAt.toIso8601String(),
  };

  factory SavedGame.fromJson(Map<String, dynamic> json) => SavedGame(
    gameId: json['gameId'] as String,
    gameName: json['gameName'] as String? ?? '',
    mode: json['mode'] as String? ?? 'four_player',
    scoring: json['scoring'] as String? ?? 'basic',
    players: Map<String, String>.from(json['players'] as Map? ?? {}),
    createdAt:
        DateTime.tryParse(json['createdAt'] as String? ?? '') ?? DateTime.now(),
  );

  // ── Equality ────────────────────────────────────────────────────────────────

  @override
  bool operator ==(Object other) =>
      other is SavedGame && other.gameId == gameId;

  @override
  int get hashCode => gameId.hashCode;
}
