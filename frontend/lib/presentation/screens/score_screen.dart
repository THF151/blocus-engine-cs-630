import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_animate/flutter_animate.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:gap/gap.dart';
import 'package:go_router/go_router.dart';

import '../../core/constants.dart';
import '../../core/router.dart';
import '../../data/models/game_state_model.dart';
import '../../domain/game_notifier.dart';
import '../../domain/providers.dart';

/// End-game score screen.
///
/// Shown automatically when the backend emits `game_finished` and the
/// [GameNotifier] transitions to [isFinished].
///
/// Displays:
/// - Ranked score table (winner highlighted).
/// - Optional advanced-scoring bonus annotations.
/// - "Play Again" and "Home" navigation buttons.
class ScoreScreen extends ConsumerStatefulWidget {
  final String gameId;
  const ScoreScreen({super.key, required this.gameId});

  @override
  ConsumerState<ScoreScreen> createState() => _ScoreScreenState();
}

class _ScoreScreenState extends ConsumerState<ScoreScreen> {
  @override
  void initState() {
    super.initState();
    // Request score in case it hasn't arrived yet
    WidgetsBinding.instance.addPostFrameCallback((_) {
      ref.read(gameNotifierProvider(widget.gameId).notifier).requestScore();
      // Remove the local game entry now that the game has ended.
      ref.read(preferencesServiceProvider).deleteGame(widget.gameId);
    });
  }

  @override
  Widget build(BuildContext context) {
    final gameState = ref.watch(gameNotifierProvider(widget.gameId));
    final report = gameState.scoreReport;
    final lobby = ref.read(lobbyNotifierProvider);

    return Scaffold(
      body: SafeArea(
        child: Center(
          child: ConstrainedBox(
            constraints: const BoxConstraints(maxWidth: 480),
            child: Padding(
              padding: const EdgeInsets.all(24),
              child: Column(
                mainAxisAlignment: MainAxisAlignment.center,
                crossAxisAlignment: CrossAxisAlignment.stretch,
                children: [
                  _TrophyHeader()
                      .animate()
                      .fadeIn(duration: 600.ms)
                      .slideY(begin: -0.3),
                  const Gap(32),
                  if (report == null)
                    const Center(child: CircularProgressIndicator())
                  else
                    _ScoreTable(
                      report: report,
                      colorToPlayerId: lobby.colorToPlayerId,
                      playerNames: lobby.playerNames,
                    ).animate().fadeIn(delay: 400.ms).slideY(begin: 0.2),
                  const Gap(40),
                  _ActionButtons(
                    gameId: widget.gameId,
                  ).animate().fadeIn(delay: 800.ms),
                ],
              ),
            ),
          ),
        ),
      ),
    );
  }
}

// ──────────────────────────────────────────────────────────────────────────────
// Sub-widgets
// ──────────────────────────────────────────────────────────────────────────────

class _TrophyHeader extends StatelessWidget {
  @override
  Widget build(BuildContext context) {
    return Column(
      children: [
        const Icon(Icons.emoji_events_rounded, size: 72, color: Colors.amber),
        const Gap(8),
        Text(
          'Game Over',
          style: Theme.of(
            context,
          ).textTheme.headlineMedium?.copyWith(fontWeight: FontWeight.bold),
          textAlign: TextAlign.center,
        ),
        Text(
          'Final Scores',
          style: Theme.of(context).textTheme.titleMedium?.copyWith(
            color: Theme.of(context).colorScheme.onSurfaceVariant,
          ),
        ),
      ],
    );
  }
}

class _ScoreTable extends StatelessWidget {
  final ScoreReportModel report;
  final Map<String, String> colorToPlayerId;
  final Map<String, String> playerNames;

  const _ScoreTable({
    required this.report,
    required this.colorToPlayerId,
    required this.playerNames,
  });

  @override
  Widget build(BuildContext context) {
    final ranked = report.ranked;
    return Card(
      child: Padding(
        padding: const EdgeInsets.all(16),
        child: Column(
          children: [
            Text(
              'Scoring: ${report.scoring}',
              style: Theme.of(context).textTheme.labelSmall,
            ),
            const Gap(12),
            ...ranked.asMap().entries.map((e) {
              final rank = e.key;
              final entry = e.value;
              // Find which colour this player_id controls
              final color =
                  colorToPlayerId.entries
                      .where((c) => c.value == entry.playerId)
                      .map((c) => c.key)
                      .firstOrNull;
              final displayName =
                  playerNames[entry.playerId] ??
                  entry.playerId.substring(
                    0,
                    entry.playerId.length.clamp(0, 8),
                  );
              return _ScoreRow(
                rank: rank + 1,
                playerId: displayName,
                score: entry.score,
                color: color,
                isWinner: rank == 0,
              ).animate(delay: (rank * 120).ms).fadeIn().slideX(begin: -0.3);
            }),
          ],
        ),
      ),
    );
  }
}

class _ScoreRow extends StatelessWidget {
  final int rank;
  final String playerId;
  final int score;
  final String? color;
  final bool isWinner;

  const _ScoreRow({
    required this.rank,
    required this.playerId,
    required this.score,
    required this.color,
    required this.isWinner,
  });

  @override
  Widget build(BuildContext context) {
    final cs = Theme.of(context).colorScheme;
    return Container(
      margin: const EdgeInsets.symmetric(vertical: 4),
      padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 10),
      decoration: BoxDecoration(
        color: isWinner ? cs.primaryContainer : cs.surfaceContainerHighest,
        borderRadius: BorderRadius.circular(10),
      ),
      child: Row(
        children: [
          // Rank badge
          SizedBox(
            width: 28,
            child: Text(
              '#$rank',
              style: TextStyle(
                fontWeight: FontWeight.bold,
                color: isWinner ? cs.onPrimaryContainer : cs.onSurface,
              ),
            ),
          ),
          // Colour dot
          if (color != null) ...[
            CircleAvatar(radius: 8, backgroundColor: colorForPlayer(color!)),
            const Gap(8),
          ],
          // Player name
          Expanded(
            child: Text(
              playerId,
              style: TextStyle(
                fontWeight: isWinner ? FontWeight.bold : FontWeight.normal,
                color: isWinner ? cs.onPrimaryContainer : cs.onSurface,
              ),
            ),
          ),
          // Score
          Text(
            '$score pts',
            style: TextStyle(
              fontWeight: FontWeight.bold,
              color: isWinner ? cs.onPrimaryContainer : cs.primary,
            ),
          ),
          if (isWinner) ...[
            const Gap(6),
            const Icon(Icons.star_rounded, color: Colors.amber, size: 18),
          ],
        ],
      ),
    );
  }
}

class _ActionButtons extends ConsumerWidget {
  final String gameId;
  const _ActionButtons({required this.gameId});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        FilledButton.icon(
          onPressed: () {
            ref.read(lobbyNotifierProvider.notifier).reset();
            ref.read(pieceSelectionProvider.notifier).clearSelection();
            context.go(kRouteHome);
          },
          icon: const Icon(Icons.home_rounded),
          label: const Text('Home'),
        ),
        const Gap(12),
        OutlinedButton.icon(
          onPressed: () {
            // Copy game ID to clipboard so the player can share it
            Clipboard.setData(ClipboardData(text: gameId));
            ScaffoldMessenger.of(context).showSnackBar(
              const SnackBar(content: Text('Game ID copied to clipboard')),
            );
          },
          icon: const Icon(Icons.copy_rounded),
          label: const Text('Copy Game ID'),
        ),
      ],
    );
  }
}
