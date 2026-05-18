/// A single legal move as returned by `request_legal_moves`.
///
/// Fields map 1-to-1 to the backend's `LegalMove` objects.
class LegalMoveModel {
  /// Engine piece ID (0–20).
  final int pieceId;

  /// Orientation index matching [PieceDefinition.orientations].
  final int orientationId;

  /// Top-left row of the piece's bounding box on the board.
  final int row;

  /// Top-left column of the piece's bounding box on the board.
  final int col;

  /// Flat board index (row * 32 + col) used internally by the engine.
  final int boardIndex;

  /// Score contribution of this placement (number of squares placed).
  final int scoreDelta;

  const LegalMoveModel({
    required this.pieceId,
    required this.orientationId,
    required this.row,
    required this.col,
    required this.boardIndex,
    required this.scoreDelta,
  });

  factory LegalMoveModel.fromJson(Map<String, dynamic> json) => LegalMoveModel(
        pieceId: json['piece_id'] as int,
        orientationId: json['orientation_id'] as int,
        row: json['row'] as int,
        col: json['col'] as int,
        boardIndex: json['board_index'] as int,
        scoreDelta: json['score_delta'] as int,
      );

  @override
  bool operator ==(Object other) =>
      other is LegalMoveModel &&
      pieceId == other.pieceId &&
      orientationId == other.orientationId &&
      row == other.row &&
      col == other.col;

  @override
  int get hashCode => Object.hash(pieceId, orientationId, row, col);
}
