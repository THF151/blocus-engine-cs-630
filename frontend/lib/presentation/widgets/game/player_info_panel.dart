import 'package:flutter/material.dart';
import 'package:flutter_animate/flutter_animate.dart';
import 'package:gap/gap.dart';

import '../../../core/constants.dart';

/// Compact card showing one player's colour, name, and remaining-cell count.
///
/// Visual states:
/// - **Inactive**: muted surface background, no border.
/// - **Active – own colour** (`currentTurnColor == color`): thick 4 px border
///   and brighter background in the player's colour.
/// - **Active – playing alternate colour** (three-player shared green):
///   outer 4 px border in the player's own colour, inner 2 px border in the
///   turn colour, plus a small secondary dot to make both roles legible.
class PlayerInfoPanel extends StatelessWidget {
  final String color;
  final String playerId;
  final int cellCount;
  final bool isActive;

  /// The colour the player is currently moving, or `null` when it is not
  /// their turn.  May differ from [color] in three-player mode when the
  /// shared green colour rotates to this player.
  final String? currentTurnColor;

  const PlayerInfoPanel({
    super.key,
    required this.color,
    required this.playerId,
    required this.cellCount,
    this.isActive = false,
    this.currentTurnColor,
  });

  @override
  Widget build(BuildContext context) {
    final playerColor = colorForPlayer(color);
    final cs = Theme.of(context).colorScheme;

    final isPlayingAltColor =
        currentTurnColor != null && currentTurnColor != color;
    final turnColor =
        isPlayingAltColor ? colorForPlayer(currentTurnColor!) : null;

    // Inner card content.
    Widget card = Container(
      padding: const EdgeInsets.symmetric(horizontal: 10, vertical: 6),
      decoration: BoxDecoration(
        color:
            isActive ? playerColor.withAlpha(60) : cs.surfaceContainerHighest,
        borderRadius: BorderRadius.circular(isPlayingAltColor ? 7 : 10),
        border:
            isActive
                ? Border.all(
                  color: isPlayingAltColor ? turnColor! : playerColor,
                  width: isPlayingAltColor ? 2 : 4,
                )
                : null,
      ),
      child: Row(
        children: [
          // Primary colour dot.
          CircleAvatar(radius: 8, backgroundColor: playerColor),
          // Secondary dot appears when playing a different colour (alt-colour mode).
          if (isPlayingAltColor) ...[
            const Gap(3),
            CircleAvatar(radius: 5, backgroundColor: turnColor),
          ],
          const Gap(6),
          Expanded(
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              mainAxisSize: MainAxisSize.min,
              children: [
                Text(
                  playerId,
                  style: Theme.of(context).textTheme.labelMedium?.copyWith(
                    fontWeight: isActive ? FontWeight.bold : FontWeight.normal,
                  ),
                  overflow: TextOverflow.ellipsis,
                  maxLines: 1,
                ),
                Text(
                  '$cellCount cells',
                  style: Theme.of(
                    context,
                  ).textTheme.labelSmall?.copyWith(color: cs.onSurfaceVariant),
                ),
              ],
            ),
          ),
        ],
      ),
    );

    // Outer border in player colour wraps the inner turn-colour border.
    if (isPlayingAltColor) {
      card = Container(
        decoration: BoxDecoration(
          borderRadius: BorderRadius.circular(10),
          border: Border.all(color: playerColor, width: 4),
        ),
        child: card,
      );
    }

    if (isActive) {
      final shimmerColor = (isPlayingAltColor ? turnColor! : playerColor)
          .withAlpha(50);
      card = card
          .animate(onPlay: (c) => c.repeat(reverse: true))
          .shimmer(duration: 1500.ms, color: shimmerColor);
    }

    return card;
  }
}
