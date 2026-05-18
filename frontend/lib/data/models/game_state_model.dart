// ──────────────────────────────────────────────────────────────────────────────
// Immutable models for the game state delivered by the backend state_view.
// ──────────────────────────────────────────────────────────────────────────────
//
// Every field maps 1-to-1 to a JSON key in the `state_view` dict produced by
// `engine_adapter.py`.  All classes are immutable and provide a [copyWith].

/// A single occupied board cell.
class BoardCellModel {
  final int row;
  final int col;
  final String color;

  const BoardCellModel({
    required this.row,
    required this.col,
    required this.color,
  });

  factory BoardCellModel.fromJson(Map<String, dynamic> json) => BoardCellModel(
    row: json['row'] as int,
    col: json['col'] as int,
    color: json['color'] as String,
  );
}

/// Occupied-cell count per colour.
class BoardCountModel {
  final String color;
  final int count;

  const BoardCountModel({required this.color, required this.count});

  factory BoardCountModel.fromJson(Map<String, dynamic> json) =>
      BoardCountModel(
        color: json['color'] as String,
        count: json['count'] as int,
      );
}

/// The full game state as seen by the frontend, derived from `state_view`.
///
/// [boardCells] contains every occupied cell so the board can be rendered
/// after reconnect without replaying history.
class GameStateModel {
  final String gameId;
  final String mode;
  final String scoring;
  final String status; // "in_progress" | "finished"
  final int version;
  final int boardSize; // 20 for Classic, 14 for Duo
  final bool boardIsEmpty;
  final String currentColor;
  final List<String> turnOrder;
  final int occupiedCount;
  final List<BoardCountModel> boardCounts;
  final List<BoardCellModel> boardCells;
  final int? sharedColorTurnIndex;

  const GameStateModel({
    required this.gameId,
    required this.mode,
    required this.scoring,
    required this.status,
    required this.version,
    required this.boardSize,
    required this.boardIsEmpty,
    required this.currentColor,
    required this.turnOrder,
    required this.occupiedCount,
    required this.boardCounts,
    required this.boardCells,
    this.sharedColorTurnIndex,
  });

  /// Parses the `state_view` JSON map sent inside backend events.
  factory GameStateModel.fromJson(Map<String, dynamic> json) => GameStateModel(
    gameId: json['game_id'] as String,
    mode: json['mode'] as String,
    scoring: json['scoring'] as String,
    status: json['status'] as String,
    version: json['version'] as int,
    boardSize: json['board_size'] as int,
    boardIsEmpty: json['board_is_empty'] as bool,
    currentColor: json['current_color'] as String,
    turnOrder: (json['turn_order'] as List<dynamic>).cast<String>(),
    occupiedCount: json['occupied_count'] as int,
    boardCounts: (json['board_counts'] as List<dynamic>)
        .map((e) => BoardCountModel.fromJson(e as Map<String, dynamic>))
        .toList(growable: false),
    boardCells:
        (json['board_cells'] as List<dynamic>?)
            ?.map((e) => BoardCellModel.fromJson(e as Map<String, dynamic>))
            .toList(growable: false) ??
        const [],
    sharedColorTurnIndex: json['shared_color_turn_index'] as int?,
  );

  /// Returns a 2-D matrix view of the board for fast cell lookup.
  ///
  /// `matrix[row][col]` is either a colour string or `null` (empty).
  List<List<String?>> toMatrix() {
    final matrix = List.generate(
      boardSize,
      (_) => List<String?>.filled(boardSize, null, growable: false),
      growable: false,
    );
    for (final cell in boardCells) {
      if (cell.row < boardSize && cell.col < boardSize) {
        matrix[cell.row][cell.col] = cell.color;
      }
    }
    return matrix;
  }

  /// Returns the occupied-cell count for a given player colour.
  int countForColor(String color) =>
      boardCounts
          .firstWhere(
            (c) => c.color == color,
            orElse: () => BoardCountModel(color: color, count: 0),
          )
          .count;

  bool get isFinished => status == 'finished';

  GameStateModel copyWith({
    String? gameId,
    String? mode,
    String? scoring,
    String? status,
    int? version,
    int? boardSize,
    bool? boardIsEmpty,
    String? currentColor,
    List<String>? turnOrder,
    int? occupiedCount,
    List<BoardCountModel>? boardCounts,
    List<BoardCellModel>? boardCells,
    int? sharedColorTurnIndex,
  }) => GameStateModel(
    gameId: gameId ?? this.gameId,
    mode: mode ?? this.mode,
    scoring: scoring ?? this.scoring,
    status: status ?? this.status,
    version: version ?? this.version,
    boardSize: boardSize ?? this.boardSize,
    boardIsEmpty: boardIsEmpty ?? this.boardIsEmpty,
    currentColor: currentColor ?? this.currentColor,
    turnOrder: turnOrder ?? this.turnOrder,
    occupiedCount: occupiedCount ?? this.occupiedCount,
    boardCounts: boardCounts ?? this.boardCounts,
    boardCells: boardCells ?? this.boardCells,
    sharedColorTurnIndex: sharedColorTurnIndex ?? this.sharedColorTurnIndex,
  );
}

// ──────────────────────────────────────────────────────────────────────────────
// Score models
// ──────────────────────────────────────────────────────────────────────────────

/// A single player's final score.
class ScoreEntryModel {
  final String playerId;
  final int score;

  const ScoreEntryModel({required this.playerId, required this.score});

  factory ScoreEntryModel.fromJson(Map<String, dynamic> json) =>
      ScoreEntryModel(
        playerId: json['player_id'] as String,
        score: json['score'] as int,
      );
}

/// The final score report for a finished game.
class ScoreReportModel {
  final String scoring; // "basic" | "advanced"
  final List<ScoreEntryModel> entries;

  const ScoreReportModel({required this.scoring, required this.entries});

  /// Entries sorted descending by score (winner first).
  List<ScoreEntryModel> get ranked {
    final sorted = List.of(entries);
    sorted.sort((a, b) => b.score.compareTo(a.score));
    return sorted;
  }

  factory ScoreReportModel.fromJson(Map<String, dynamic> json) =>
      ScoreReportModel(
        scoring: json['scoring'] as String,
        entries: (json['entries'] as List<dynamic>)
            .map((e) => ScoreEntryModel.fromJson(e as Map<String, dynamic>))
            .toList(growable: false),
      );
}
