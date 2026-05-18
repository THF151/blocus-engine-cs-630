import 'package:flutter/material.dart';
import 'package:flutter_animate/flutter_animate.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../../../core/constants.dart';
import '../../../domain/providers.dart';
import 'piece_painter.dart';

/// Visual representation of a single Blokus piece in the tray.
///
/// Features:
/// - Draws the piece using [PiecePainter] (CustomPaint).
/// - Tapping selects / deselects the piece via [PieceSelectionNotifier].
/// - Used pieces are rendered with reduced opacity and without interaction.
/// - Wraps the content in a [Draggable<int>] (data = pieceId) for
///   drag-to-board placement.
class PieceWidget extends ConsumerWidget {
  final int pieceId;
  final String playerColor;
  final bool isUsed;

  /// Size of one cell in logical pixels — should match the game board's cell
  /// size so that every piece is displayed at the same scale as the board.
  final double cellSize;

  const PieceWidget({
    super.key,
    required this.pieceId,
    required this.playerColor,
    required this.cellSize,
    this.isUsed = false,
  });

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final sel = ref.watch(pieceSelectionProvider);
    final isSelected = sel.selectedPieceId == pieceId;
    final orientationIndex = isSelected ? sel.orientationIndex : 0;

    final piece = pieceById(pieceId);
    final cells = piece.cellsForOrientation(orientationIndex);
    final baseColor = colorForPlayer(playerColor);
    final displayColor =
        isUsed ? baseColor.withAlpha(70) : baseColor;

    // Compute bounding box so the widget is sized exactly to the piece.
    int minR = cells[0].$1, maxR = cells[0].$1;
    int minC = cells[0].$2, maxC = cells[0].$2;
    for (final c in cells) {
      if (c.$1 < minR) minR = c.$1;
      if (c.$1 > maxR) maxR = c.$1;
      if (c.$2 < minC) minC = c.$2;
      if (c.$2 > maxC) maxC = c.$2;
    }
    final pw = (maxC - minC + 1) * cellSize; // pixel width of piece
    final ph = (maxR - minR + 1) * cellSize; // pixel height of piece

    // ── Visual wrapper ─────────────────────────────────────────────────────

    Widget pieceVisual = CustomPaint(
      size: Size(pw, ph),
      painter: PiecePainter(
        cells: cells,
        color: displayColor,
        selected: isSelected,
        fixedCellSize: cellSize,
      ),
    );

    if (isSelected) {
      pieceVisual = Container(
        decoration: BoxDecoration(
          borderRadius: BorderRadius.circular(8),
          border: Border.all(
            color: Theme.of(context).colorScheme.primary,
            width: 2,
          ),
        ),
        child: pieceVisual,
      ).animate(onPlay: (c) => c.repeat(reverse: true)).shimmer(
            duration: 800.ms,
            color: Theme.of(context).colorScheme.primary.withAlpha(80),
          );
    }

    if (isUsed) {
      return SizedBox(
        width: pw,
        height: ph,
        child: Opacity(opacity: 0.35, child: pieceVisual),
      );
    }

    // ── Draggable + tap ────────────────────────────────────────────────────

    return GestureDetector(
      onTap: () {
        ref.read(pieceSelectionProvider.notifier).selectPiece(pieceId);
      },
      child: Draggable<int>(
        data: pieceId,
        onDragStarted: () {
          // Ensure the piece is selected when dragging starts so the
          // orientation is known to the board's DragTarget.
          ref.read(pieceSelectionProvider.notifier).selectPiece(pieceId);
        },
        feedback: Material(
          color: Colors.transparent,
          child: Opacity(
            opacity: 0.85,
            child: CustomPaint(
              size: Size(pw * 1.15, ph * 1.15),
              painter: PiecePainter(
                cells: cells,
                color: baseColor,
                fixedCellSize: cellSize * 1.15,
              ),
            ),
          ),
        ),
        childWhenDragging: Opacity(
          opacity: 0.3,
          child: pieceVisual,
        ),
        child: SizedBox(width: pw, height: ph, child: pieceVisual),
      ),
    );
  }
}
