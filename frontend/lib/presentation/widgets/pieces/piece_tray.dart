import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../../../domain/lobby_notifier.dart';
import '../../../domain/providers.dart';
import 'piece_widget.dart';

/// Vertical scrollable list showing all 21 Blokus pieces for the local
/// player.
///
/// - Pieces are sorted: available first, used last.
/// - Used pieces are dimmed and non-interactive.
/// - The tray is only interactive when it is the local player's turn.
class PieceTray extends ConsumerWidget {
  final String gameId;

  const PieceTray({super.key, required this.gameId});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final gs = ref.watch(gameNotifierProvider(gameId));
    final lobby = ref.read(lobbyNotifierProvider);

    final currentColor = gs.gameState?.currentColor ?? 'blue';
    final isMyTurn =
        gs.gameState != null &&
        lobby.localColors.contains(currentColor) &&
        isHumanControlledTurn(
          mode: lobby.mode,
          currentColor: currentColor,
          sharedColorTurnIndex: gs.gameState?.sharedColorTurnIndex,
        );

    // Determine the colour to render for local player's pieces.
    // If the local player controls multiple colours (2-player mode), use the
    // current turn colour when it is their turn, otherwise fall back to the
    // first local colour.
    final displayColor =
        isMyTurn
            ? currentColor
            : (lobby.localColors.isNotEmpty ? lobby.localColors.first : 'blue');

    final usedIds = gs.usedPieceIds;

    // Only show available pieces – used pieces are removed from the list.
    final availableIds =
        List.generate(
          21,
          (i) => i,
        ).where((id) => !usedIds.contains(id)).toList();

    return IgnorePointer(
      ignoring: !isMyTurn,
      child: Opacity(
        opacity: isMyTurn ? 1.0 : 0.5,
        child: LayoutBuilder(
          builder: (context, constraints) {
            // Derive cell size from the available tray height so the tallest
            // piece (5 cells) always fits without overflowing.
            final trayCellSize = ((constraints.maxHeight - 16) / 5).clamp(
              8.0,
              28.0,
            );
            return ListView.builder(
              scrollDirection: Axis.horizontal,
              padding: const EdgeInsets.symmetric(horizontal: 8),
              itemCount: availableIds.length,
              itemBuilder: (context, index) {
                final id = availableIds[index];
                return Padding(
                  padding: const EdgeInsets.symmetric(
                    horizontal: 4,
                    vertical: 8,
                  ),
                  child: PieceWidget(
                    pieceId: id,
                    playerColor: displayColor,
                    cellSize: trayCellSize,
                    isUsed: false,
                  ),
                );
              },
            );
          },
        ),
      ),
    );
  }
}
