import 'dart:math' as math;

import 'package:flutter/material.dart';

import '../../../core/constants.dart';

/// [CustomPainter] used by [PieceWidget] and drag feedback.
///
/// Draws one Blokus piece in the given orientation + colour.
/// The piece is fitted inside [size] with equal padding on all sides.
class PiecePainter extends CustomPainter {
  final OrientationCells cells;
  final Color color;
  final bool selected;

  /// When set, each piece cell is drawn at exactly this size (matching board
  /// cell size). When null, the piece is fitted inside the canvas with padding.
  final double? fixedCellSize;

  const PiecePainter({
    required this.cells,
    required this.color,
    this.selected = false,
    this.fixedCellSize,
  });

  @override
  void paint(Canvas canvas, Size size) {
    if (cells.isEmpty) return;

    // Compute bounding box of the piece
    int minR = cells[0].$1, maxR = cells[0].$1;
    int minC = cells[0].$2, maxC = cells[0].$2;
    for (final c in cells) {
      minR = math.min(minR, c.$1);
      maxR = math.max(maxR, c.$1);
      minC = math.min(minC, c.$2);
      maxC = math.max(maxC, c.$2);
    }
    final pieceRows = (maxR - minR + 1).toDouble();
    final pieceCols = (maxC - minC + 1).toDouble();

    final double cellSize;
    final double offsetX;
    final double offsetY;
    if (fixedCellSize != null) {
      cellSize = fixedCellSize!;
      offsetX = 0;
      offsetY = 0;
    } else {
      final padding = size.shortestSide * 0.08;
      final available = Size(
        size.width - 2 * padding,
        size.height - 2 * padding,
      );
      cellSize = math.min(
        available.width / pieceCols,
        available.height / pieceRows,
      );
      offsetX = padding + (available.width - pieceCols * cellSize) / 2;
      offsetY = padding + (available.height - pieceRows * cellSize) / 2;
    }

    final fillPaint = Paint()..color = color;
    final borderPaint =
        Paint()
          ..color = Colors.black.withAlpha(60)
          ..style = PaintingStyle.stroke
          ..strokeWidth = 0.8;

    for (final (r, c) in cells) {
      final rect = Rect.fromLTWH(
        offsetX + (c - minC) * cellSize,
        offsetY + (r - minR) * cellSize,
        cellSize,
        cellSize,
      );
      canvas.drawRect(rect.deflate(0.5), fillPaint);
      canvas.drawRect(rect.deflate(0.5), borderPaint);
    }

    // Selection glow
    if (selected) {
      final glowPaint =
          Paint()
            ..color = Colors.white.withAlpha(120)
            ..style = PaintingStyle.stroke
            ..strokeWidth = 2.5;
      for (final (r, c) in cells) {
        final rect = Rect.fromLTWH(
          offsetX + (c - minC) * cellSize,
          offsetY + (r - minR) * cellSize,
          cellSize,
          cellSize,
        );
        canvas.drawRect(rect.deflate(1), glowPaint);
      }
    }
  }

  @override
  bool shouldRepaint(PiecePainter old) =>
      cells != old.cells ||
      color != old.color ||
      selected != old.selected ||
      fixedCellSize != old.fixedCellSize;
}
