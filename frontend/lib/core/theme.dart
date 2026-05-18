import 'package:flutter/material.dart';
import 'constants.dart';

// ──────────────────────────────────────────────────────────────────────────────
// ThemeExtension – exposes player colours to any widget in the tree
// ──────────────────────────────────────────────────────────────────────────────

/// Custom theme extension that gives widgets type-safe access to the six
/// Blokus player colours without relying on raw constants.
///
/// Usage:
/// ```dart
/// Theme.of(context).extension<PlayerColors>()!.forPlayer('blue')
/// ```
class PlayerColors extends ThemeExtension<PlayerColors> {
  final Color blue;
  final Color yellow;
  final Color red;
  final Color green;
  final Color black;
  final Color white;

  const PlayerColors({
    required this.blue,
    required this.yellow,
    required this.red,
    required this.green,
    required this.black,
    required this.white,
  });

  /// Returns the colour for the given backend colour string.
  Color forPlayer(String name) {
    return switch (name) {
      'blue' => blue,
      'yellow' => yellow,
      'red' => red,
      'green' => green,
      'black' => black,
      'white' => white,
      _ => const Color(0xFF9E9E9E),
    };
  }

  @override
  PlayerColors copyWith({
    Color? blue,
    Color? yellow,
    Color? red,
    Color? green,
    Color? black,
    Color? white,
  }) => PlayerColors(
    blue: blue ?? this.blue,
    yellow: yellow ?? this.yellow,
    red: red ?? this.red,
    green: green ?? this.green,
    black: black ?? this.black,
    white: white ?? this.white,
  );

  @override
  PlayerColors lerp(PlayerColors? other, double t) {
    if (other == null) return this;
    return PlayerColors(
      blue: Color.lerp(blue, other.blue, t)!,
      yellow: Color.lerp(yellow, other.yellow, t)!,
      red: Color.lerp(red, other.red, t)!,
      green: Color.lerp(green, other.green, t)!,
      black: Color.lerp(black, other.black, t)!,
      white: Color.lerp(white, other.white, t)!,
    );
  }
}

// ──────────────────────────────────────────────────────────────────────────────
// AppTheme
// ──────────────────────────────────────────────────────────────────────────────

/// Factory for the application's Material Design 3 themes.
abstract final class AppTheme {
  static const _playerColorsLight = PlayerColors(
    blue: Color(0xFF1565C0),
    yellow: Color(0xFFFFB300),
    red: Color(0xFFC62828),
    green: Color(0xFF2E7D32),
    black: Color(0xFF212121),
    white: Color(0xFFEEEEEE),
  );

  static const _playerColorsDark = PlayerColors(
    blue: Color(0xFF64B5F6),
    yellow: Color(0xFFFFD54F),
    red: Color(0xFFEF9A9A),
    green: Color(0xFF81C784),
    black: Color(0xFF757575),
    white: Color(0xFFF5F5F5),
  );

  /// Light theme – seeded from the Blokus classic blue.
  static ThemeData light() => ThemeData(
    useMaterial3: true,
    colorScheme: ColorScheme.fromSeed(
      seedColor: kPlayerColors['blue']!,
      brightness: Brightness.light,
    ),
    extensions: const [_playerColorsLight],
  );

  /// Dark theme – same seed, dark brightness.
  static ThemeData dark() => ThemeData(
    useMaterial3: true,
    colorScheme: ColorScheme.fromSeed(
      seedColor: kPlayerColors['blue']!,
      brightness: Brightness.dark,
    ),
    extensions: const [_playerColorsDark],
  );
}
