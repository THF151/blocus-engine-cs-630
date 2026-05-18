import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:gap/gap.dart';

import '../../../core/constants.dart';
import '../../../domain/providers.dart';
import 'piece_painter.dart';

/// Shows the currently selected piece in its active orientation with buttons
/// to rotate through all available orientations.
///
/// Hidden when no piece is selected.
class PieceOrientationSelector extends ConsumerWidget {
  const PieceOrientationSelector({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final sel = ref.watch(pieceSelectionProvider);
    if (!sel.hasPieceSelected) return const SizedBox.shrink();

    final notifier = ref.read(pieceSelectionProvider.notifier);
    final cells = sel.currentCells!;
    final totalOrientations = sel.orientationCount;
    final currentIdx = sel.orientationIndex;

    // Find out which colour is active
    // We don't have gameId here so read from lobby → current or first local colour.
    final lobby = ref.read(lobbyNotifierProvider);
    final color = colorForPlayer(
      lobby.localColors.isNotEmpty ? lobby.localColors.first : 'blue',
    );

    return Container(
      height: 72,
      margin: const EdgeInsets.symmetric(horizontal: 8, vertical: 4),
      decoration: BoxDecoration(
        color: Theme.of(context).colorScheme.surfaceContainerHighest,
        borderRadius: BorderRadius.circular(12),
      ),
      child: FittedBox(
        fit: BoxFit.scaleDown,
        child: Row(
          mainAxisAlignment: MainAxisAlignment.center,
          children: [
            // Prev orientation
            IconButton(
              icon: const Icon(Icons.rotate_left_rounded),
              tooltip: 'Previous orientation',
              onPressed: () => notifier.rotatePrev(),
            ),
            // Piece preview
            RepaintBoundary(
              child: SizedBox(
                width: 56,
                height: 56,
                child: CustomPaint(
                  painter: PiecePainter(cells: cells, color: color),
                ),
              ),
            ),
            const Gap(8),
            // Orientation counter
            Text(
              '${currentIdx + 1}/$totalOrientations',
              style: Theme.of(context).textTheme.labelMedium,
            ),
            // Next orientation
            IconButton(
              icon: const Icon(Icons.rotate_right_rounded),
              tooltip: 'Next orientation',
              onPressed: () => notifier.rotateNext(),
            ),
            // Deselect
            IconButton(
              icon: const Icon(Icons.close_rounded),
              tooltip: 'Deselect piece',
              onPressed: () => notifier.clearSelection(),
            ),
          ],
        ),
      ),
    );
  }
}
