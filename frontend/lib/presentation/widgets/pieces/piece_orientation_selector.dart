import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:gap/gap.dart';

import '../../../core/constants.dart';
import '../../../domain/providers.dart';
import 'piece_painter.dart';

/// Full-width horizontal bar shown above the piece tray when a piece is
/// selected.
///
/// Collapses to [SizedBox.shrink] when nothing is selected.  The caller
/// wraps this in an [AnimatedSize] so the bar slides in and out without
/// causing layout overflows.
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

    final lobby = ref.read(lobbyNotifierProvider);
    final color = colorForPlayer(
      lobby.localColors.isNotEmpty ? lobby.localColors.first : 'blue',
    );

    final cs = Theme.of(context).colorScheme;

    return Container(
      height: 64,
      decoration: BoxDecoration(
        color: cs.surfaceContainerHighest,
        border: Border(bottom: BorderSide(color: cs.outlineVariant, width: 1)),
      ),
      child: Row(
        mainAxisAlignment: MainAxisAlignment.center,
        children: [
          IconButton(
            icon: const Icon(Icons.rotate_left_rounded),
            iconSize: 20,
            visualDensity: VisualDensity.compact,
            tooltip: 'Previous orientation',
            onPressed: () => notifier.rotatePrev(),
          ),
          // Piece preview centred inside a fixed square.
          RepaintBoundary(
            child: SizedBox(
              width: 48,
              height: 48,
              child: CustomPaint(
                painter: PiecePainter(cells: cells, color: color),
              ),
            ),
          ),
          IconButton(
            icon: const Icon(Icons.rotate_right_rounded),
            iconSize: 20,
            visualDensity: VisualDensity.compact,
            tooltip: 'Next orientation',
            onPressed: () => notifier.rotateNext(),
          ),
          Text(
            '${currentIdx + 1} / $totalOrientations',
            style: Theme.of(context).textTheme.labelMedium,
          ),
          const Gap(16),
          IconButton(
            icon: const Icon(Icons.close_rounded),
            iconSize: 20,
            visualDensity: VisualDensity.compact,
            tooltip: 'Deselect piece',
            onPressed: () => notifier.clearSelection(),
          ),
        ],
      ),
    );
  }
}
