import 'package:flutter/material.dart';

import '../../../core/constants.dart';

/// [CustomPainter] that renders the entire Blokus board in a single paint call.
///
/// All rendering is done on the [Canvas] directly – no per-cell widgets.
/// This keeps the widget tree flat and avoids expensive rebuilds for each
/// of the up to 400 cells on a 20×20 board.
///
/// Inputs:
/// - [boardMatrix]: `[row][col]` → colour string or null (empty).
/// - [boardSize]: 14 for Duo, 20 for Classic.
/// - [startCorners]: cells to be rendered with a distinct marker.
/// - [legalPositions]: top-left anchors of valid placements for the selected
///   piece+orientation (highlighted with a translucent fill).
/// - [previewCells]: absolute (row, col) cells the selected piece would occupy
///   at the current hover / drag position.
/// - [activeColor]: the current player's colour string (used for highlights).
class GameBoardPainter extends CustomPainter {
  final List<List<String?>> boardMatrix;
  final int boardSize;
  final Set<(int, int)> startCorners;
  final Set<(int, int)> legalPositions;
  final List<(int, int)>? previewCells;
  final String activeColor;

  // ── Cached paints (initialized in constructor) ────────────────────────────

  late Paint _gridPaint;
  late Paint _emptyPaint;
  late Paint _cornerPaint;
  late Paint _highlightPaint;
  late Paint _previewPaint;

  // Corner triangle size relative to cell size
  static const double _cornerFraction = 0.25;

  GameBoardPainter({
    required this.boardMatrix,
    required this.boardSize,
    required this.startCorners,
    required this.legalPositions,
    this.previewCells,
    required this.activeColor,
  }) {
    _gridPaint =
        Paint()
          ..color = const Color(0xFFBDBDBD)
          ..strokeWidth = 0.5
          ..style = PaintingStyle.stroke;
    _emptyPaint = Paint()..color = const Color(0xFFF5F5F5);
    _cornerPaint = Paint()..color = const Color(0xFFE0E0E0);
    _highlightPaint =
        Paint()..color = colorForPlayer(activeColor).withAlpha(80);
    _previewPaint = Paint()..color = colorForPlayer(activeColor).withAlpha(160);
  }

  @override
  void paint(Canvas canvas, Size size) {
    final cellSize = size.width / boardSize;

    // ── 1. Background pass ─────────────────────────────────────────────────
    for (int r = 0; r < boardSize; r++) {
      for (int c = 0; c < boardSize; c++) {
        final rect = _cellRect(r, c, cellSize);
        final color = boardMatrix[r][c];

        if (color != null) {
          // Occupied cell
          final paint = Paint()..color = colorForPlayer(color);
          canvas.drawRect(rect, paint);
          // Subtle inner border for visual separation between same-colour tiles
          final innerPaint =
              Paint()
                ..color = Colors.black.withAlpha(30)
                ..style = PaintingStyle.stroke
                ..strokeWidth = 0.8;
          canvas.drawRect(rect.deflate(1), innerPaint);
        } else if (startCorners.contains((r, c))) {
          // Start corner – slightly tinted background
          canvas.drawRect(rect, _cornerPaint);
          _drawCornerMarker(canvas, rect);
        } else {
          canvas.drawRect(rect, _emptyPaint);
        }
      }
    }

    // ── 2. Legal-move highlights ───────────────────────────────────────────
    for (final pos in legalPositions) {
      final rect = _cellRect(pos.$1, pos.$2, cellSize);
      canvas.drawRect(rect, _highlightPaint);
    }

    // ── 3. Piece preview during drag / hover ──────────────────────────────
    if (previewCells != null) {
      for (final cell in previewCells!) {
        final r = cell.$1;
        final c = cell.$2;
        if (r < 0 || r >= boardSize || c < 0 || c >= boardSize) continue;
        final rect = _cellRect(r, c, cellSize).deflate(1);
        // Draw with rounded corners for a "floating" look
        canvas.drawRRect(
          RRect.fromRectAndRadius(rect, const Radius.circular(3)),
          _previewPaint,
        );
      }
    }

    // ── 4. Grid lines ──────────────────────────────────────────────────────
    for (int r = 0; r <= boardSize; r++) {
      final y = r * cellSize;
      canvas.drawLine(Offset(0, y), Offset(size.width, y), _gridPaint);
    }
    for (int c = 0; c <= boardSize; c++) {
      final x = c * cellSize;
      canvas.drawLine(Offset(x, 0), Offset(x, size.height), _gridPaint);
    }
  }

  // ── Helpers ───────────────────────────────────────────────────────────────

  Rect _cellRect(int row, int col, double cellSize) =>
      Rect.fromLTWH(col * cellSize, row * cellSize, cellSize, cellSize);

  /// Draws a small diagonal triangle in the corner of a start-corner cell.
  void _drawCornerMarker(Canvas canvas, Rect rect) {
    final markerPaint = Paint()..color = const Color(0xFFBDBDBD);
    final s = rect.width * _cornerFraction;
    // Determine which corner based on the cell position – draw towards center
    final path =
        Path()
          ..moveTo(rect.left, rect.top)
          ..lineTo(rect.left + s, rect.top)
          ..lineTo(rect.left, rect.top + s)
          ..close();
    canvas.drawPath(path, markerPaint);
  }

  @override
  bool shouldRepaint(GameBoardPainter old) =>
      !identical(boardMatrix, old.boardMatrix) ||
      legalPositions != old.legalPositions ||
      previewCells != old.previewCells ||
      activeColor != old.activeColor ||
      boardSize != old.boardSize;
}
