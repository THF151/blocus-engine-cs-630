import 'dart:math' as math;
import 'package:flutter/material.dart';

// ──────────────────────────────────────────────────────────────────────────────
// Board dimensions
// ──────────────────────────────────────────────────────────────────────────────

/// Classic / Two-Player / Three-Player / Four-Player board size.
const int kClassicBoardSize = 20;

/// Duo board size.
const int kDuoBoardSize = 14;

// ──────────────────────────────────────────────────────────────────────────────
// Game-mode / colour string constants (match backend JSON values exactly)
// ──────────────────────────────────────────────────────────────────────────────

const String kModeTwo = 'two_player';
const String kModeThree = 'three_player';
const String kModeFour = 'four_player';
const String kModeDuo = 'duo';

const String kScoringBasic = 'basic';
const String kScoringAdvanced = 'advanced';

const String kStatusInProgress = 'in_progress';
const String kStatusFinished = 'finished';

/// All classic-mode colours in their canonical turn order.
const List<String> kClassicColors = ['blue', 'yellow', 'red', 'green'];

/// All duo-mode colours.
const List<String> kDuoColors = ['black', 'white'];

// ──────────────────────────────────────────────────────────────────────────────
// Start-corner positions
// ──────────────────────────────────────────────────────────────────────────────

/// Classic start-corner positions keyed by colour.
///
/// Each player must place their first piece touching this corner.
const Map<String, (int, int)> kClassicStartCorners = {
  'blue': (0, 0),
  'yellow': (0, 19),
  'red': (19, 19),
  'green': (19, 0),
};

/// Duo start positions (Black = inner top-left, White = inner bottom-right).
const Map<String, (int, int)> kDuoStartCorners = {
  'black': (4, 4),
  'white': (9, 9),
};

/// Returns the start-corner set for the given mode string.
Set<(int, int)> startCornersForMode(String mode) {
  if (mode == kModeDuo) {
    return kDuoStartCorners.values.toSet();
  }
  return kClassicStartCorners.values.toSet();
}

// ──────────────────────────────────────────────────────────────────────────────
// Player-colour UI mapping
// ──────────────────────────────────────────────────────────────────────────────

/// Maps each backend colour string to a Flutter [Color].
const Map<String, Color> kPlayerColors = {
  'blue': Color(0xFF1565C0),
  'yellow': Color(0xFFFFB300),
  'red': Color(0xFFC62828),
  'green': Color(0xFF2E7D32),
  'black': Color(0xFF212121),
  'white': Color(0xFFEEEEEE),
};

/// Returns the [Color] for a given colour string, defaulting to grey.
Color colorForPlayer(String colorName) =>
    kPlayerColors[colorName] ?? const Color(0xFF9E9E9E);

// ──────────────────────────────────────────────────────────────────────────────
// Piece definitions
// ──────────────────────────────────────────────────────────────────────────────

/// A single piece orientation: normalised (0,0)-origin cell list.
///
/// Cells are expressed as `(row, col)` records, sorted top-left to
/// bottom-right for deterministic equality checks.
typedef OrientationCells = List<(int, int)>;

/// Complete descriptor for one of the 21 Blokus pieces.
class PieceDefinition {
  /// Engine-assigned ID (0–20, matches `piece_id` in the protocol).
  final int id;

  /// Human-readable piece name (e.g. "I5", "X5").
  final String name;

  /// All unique orientations precomputed with the same algorithm as the
  /// Rust engine.  The index into this list IS the `orientation_id` used
  /// in the WebSocket protocol.
  final List<OrientationCells> orientations;

  const PieceDefinition({
    required this.id,
    required this.name,
    required this.orientations,
  });

  /// Returns the bounding-box width of a given orientation (max col + 1).
  int widthFor(int orientationId) {
    final cells = orientations[orientationId.clamp(0, orientations.length - 1)];
    return cells.fold(0, (w, c) => math.max(w, c.$2 + 1));
  }

  /// Returns the bounding-box height of a given orientation (max row + 1).
  int heightFor(int orientationId) {
    final cells = orientations[orientationId.clamp(0, orientations.length - 1)];
    return cells.fold(0, (h, c) => math.max(h, c.$1 + 1));
  }

  /// Returns the cells for [orientationId], falling back to orientation 0.
  OrientationCells cellsForOrientation(int orientationId) =>
      orientations[orientationId.clamp(0, orientations.length - 1)];

  /// Returns the absolute board cells when this piece (in [orientationId]) is
  /// placed with its bounding-box top-left at board coordinate ([row], [col]).
  List<(int, int)> absoluteCells(int orientationId, int row, int col) =>
      cellsForOrientation(
        orientationId,
      ).map((c) => (c.$1 + row, c.$2 + col)).toList();
}

// ──────────────────────────────────────────────────────────────────────────────
// Orientation computation – must match the Rust engine exactly
// ──────────────────────────────────────────────────────────────────────────────

/// Computes all unique orientations for a piece given its base cells.
///
/// Iterates: flip ∈ {none, horizontal} × rotation ∈ {0°, 90°, 180°, 270° CW}
/// and deduplicates by a normalised canonical key — matching the Rust engine's
/// `UniqueOrientations::from_base` algorithm so that `orientation_id` indices
/// are identical on both sides of the protocol.
List<OrientationCells> computeOrientations(List<(int, int)> baseCells) {
  final result = <OrientationCells>[];
  final seen = <String>{};

  for (final flip in [false, true]) {
    for (int rot = 0; rot < 4; rot++) {
      var cells = List.of(baseCells);

      // Horizontal flip: (r, c) → (r, -c), then normalise.
      if (flip) {
        cells = cells.map((c) => (c.$1, -c.$2)).toList();
      }

      // Rotate 90° CW applied [rot] times: (r, c) → (c, -r).
      for (int i = 0; i < rot; i++) {
        cells = cells.map((c) => (c.$2, -c.$1)).toList();
      }

      // Normalise: shift so the minimum row and col are both 0.
      cells = _normalise(cells);

      // Sort for a deterministic canonical key.
      cells.sort((a, b) {
        final dr = a.$1 - b.$1;
        return dr != 0 ? dr : a.$2 - b.$2;
      });

      final key = cells.map((c) => '${c.$1}:${c.$2}').join(',');
      if (seen.add(key)) {
        result.add(cells);
      }
    }
  }
  return result;
}

List<(int, int)> _normalise(List<(int, int)> cells) {
  final minR = cells.fold(cells[0].$1, (m, c) => c.$1 < m ? c.$1 : m);
  final minC = cells.fold(cells[0].$2, (m, c) => c.$2 < m ? c.$2 : m);
  return cells.map((c) => (c.$1 - minR, c.$2 - minC)).toList();
}

// ──────────────────────────────────────────────────────────────────────────────
// All 21 piece definitions (IDs and base shapes from blocus-core/pieces/repository.rs)
// ──────────────────────────────────────────────────────────────────────────────

/// The complete, ordered list of all 21 Blokus pieces.
///
/// Indexed by `piece_id` (0–20). Orientations are precomputed at app startup.
final List<PieceDefinition> kPieces = _buildPieces();

List<PieceDefinition> _buildPieces() {
  // Each entry: (id, name, baseCells)
  final raw = <(int, String, List<(int, int)>)>[
    (0, 'I1', [(0, 0)]),
    (1, 'I2', [(0, 0), (0, 1)]),
    (2, 'I3', [(0, 0), (0, 1), (0, 2)]),
    (3, 'V3', [(0, 0), (1, 0), (1, 1)]),
    (4, 'I4', [(0, 0), (0, 1), (0, 2), (0, 3)]),
    (5, 'O4', [(0, 0), (0, 1), (1, 0), (1, 1)]),
    (6, 'T4', [(0, 0), (0, 1), (0, 2), (1, 1)]),
    (7, 'L4', [(0, 0), (1, 0), (2, 0), (2, 1)]),
    (8, 'Z4', [(0, 0), (0, 1), (1, 1), (1, 2)]),
    (9, 'F5', [(0, 1), (1, 0), (1, 1), (1, 2), (2, 2)]),
    (10, 'I5', [(0, 0), (0, 1), (0, 2), (0, 3), (0, 4)]),
    (11, 'L5', [(0, 0), (1, 0), (2, 0), (3, 0), (3, 1)]),
    (12, 'N5', [(0, 1), (1, 1), (2, 0), (2, 1), (3, 0)]),
    (13, 'P5', [(0, 0), (0, 1), (1, 0), (1, 1), (2, 0)]),
    (14, 'T5', [(0, 0), (0, 1), (0, 2), (1, 1), (2, 1)]),
    (15, 'U5', [(0, 0), (0, 2), (1, 0), (1, 1), (1, 2)]),
    (16, 'V5', [(0, 0), (1, 0), (2, 0), (2, 1), (2, 2)]),
    (17, 'W5', [(0, 0), (1, 0), (1, 1), (2, 1), (2, 2)]),
    (18, 'X5', [(0, 1), (1, 0), (1, 1), (1, 2), (2, 1)]),
    (19, 'Y5', [(0, 0), (1, 0), (2, 0), (3, 0), (2, 1)]),
    (20, 'Z5', [(0, 0), (0, 1), (1, 1), (2, 1), (2, 2)]),
  ];

  return raw
      .map((t) {
        final (id, name, base) = t;
        return PieceDefinition(
          id: id,
          name: name,
          orientations: computeOrientations(base),
        );
      })
      .toList(growable: false);
}

/// Lookup a [PieceDefinition] by its engine ID.
PieceDefinition pieceById(int id) => kPieces[id.clamp(0, kPieces.length - 1)];
