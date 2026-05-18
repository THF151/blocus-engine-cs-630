import 'package:flutter/material.dart';
import 'package:flutter_animate/flutter_animate.dart';
import 'package:gap/gap.dart';

import '../../../core/constants.dart';

/// Compact card showing one player's colour, name, and remaining-cell count.
class PlayerInfoPanel extends StatelessWidget {
  final String color;
  final String playerId;
  final int cellCount;
  final bool isActive;

  const PlayerInfoPanel({
    super.key,
    required this.color,
    required this.playerId,
    required this.cellCount,
    this.isActive = false,
  });

  @override
  Widget build(BuildContext context) {
    final playerColor = colorForPlayer(color);
    final cs = Theme.of(context).colorScheme;

    Widget card = Container(
      padding: const EdgeInsets.symmetric(horizontal: 10, vertical: 6),
      decoration: BoxDecoration(
        color: isActive ? playerColor.withAlpha(40) : cs.surfaceContainerHighest,
        borderRadius: BorderRadius.circular(10),
        border: isActive
            ? Border.all(color: playerColor, width: 2)
            : null,
      ),
      child: Row(
        children: [
          CircleAvatar(
            radius: 8,
            backgroundColor: playerColor,
          ),
          const Gap(6),
          Column(
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
                style: Theme.of(context).textTheme.labelSmall?.copyWith(
                      color: cs.onSurfaceVariant,
                    ),
              ),
            ],
          ),
        ],
      ),
    );

    if (isActive) {
      card = card
          .animate(onPlay: (c) => c.repeat(reverse: true))
          .shimmer(duration: 1500.ms, color: playerColor.withAlpha(50));
    }

    return card;
  }
}
