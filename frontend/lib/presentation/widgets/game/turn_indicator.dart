import 'package:flutter/material.dart';
import 'package:flutter_animate/flutter_animate.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:gap/gap.dart';

import '../../../core/constants.dart';
import '../../../domain/providers.dart';

/// Animated banner showing whose turn it is.
///
/// Pulses when it is the local player's turn to draw attention.
class TurnIndicator extends ConsumerWidget {
  final String gameId;
  const TurnIndicator({super.key, required this.gameId});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final gs = ref.watch(gameNotifierProvider(gameId));
    final lobby = ref.read(lobbyNotifierProvider);

    final currentColor = gs.gameState?.currentColor;
    if (currentColor == null) return const SizedBox.shrink();

    final playerColor = colorForPlayer(currentColor);
    final playerId = lobby.colorToPlayerId[currentColor] ?? currentColor;
    final displayName =
        lobby.playerNames[playerId] ??
        playerId.substring(0, playerId.length.clamp(0, 8));
    final isMe = lobby.localColors.contains(currentColor);

    Widget indicator = Container(
      margin: const EdgeInsets.symmetric(horizontal: 16, vertical: 4),
      padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 8),
      decoration: BoxDecoration(
        color: playerColor.withAlpha(isMe ? 50 : 25),
        borderRadius: BorderRadius.circular(24),
        border: Border.all(color: playerColor, width: 1.5),
      ),
      child: FittedBox(
        fit: BoxFit.scaleDown,
        child: Row(
          mainAxisSize: MainAxisSize.min,
          children: [
            CircleAvatar(radius: 6, backgroundColor: playerColor),
            const Gap(8),
            Text(
              isMe ? 'Your turn!' : '$displayName\'s turn',
              style: Theme.of(context).textTheme.labelLarge?.copyWith(
                color: playerColor,
                fontWeight: FontWeight.bold,
              ),
            ),
            if (gs.isLoadingMove) ...[
              const Gap(8),
              SizedBox(
                width: 12,
                height: 12,
                child: CircularProgressIndicator(
                  strokeWidth: 2,
                  color: playerColor,
                ),
              ),
            ],
          ],
        ),
      ),
    );

    if (isMe) {
      indicator = indicator
          .animate(onPlay: (c) => c.repeat(reverse: true))
          .scale(
            begin: const Offset(1.0, 1.0),
            end: const Offset(1.03, 1.03),
            duration: 700.ms,
            curve: Curves.easeInOut,
          );
    }

    return indicator;
  }
}
