import 'package:flutter/material.dart';
import 'package:flutter_animate/flutter_animate.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:gap/gap.dart';
import 'package:go_router/go_router.dart';

import '../../core/router.dart';
import '../../domain/game_notifier.dart';
import '../../domain/lobby_notifier.dart';
import '../../domain/providers.dart';
import '../widgets/board/game_board.dart';
import '../widgets/game/player_info_panel.dart';
import '../widgets/pieces/piece_orientation_selector.dart';
import '../widgets/pieces/piece_tray.dart';
import '../widgets/shared/connection_status_banner.dart';

/// The main in-game screen.
///
/// Layout (responsive):
/// - **Mobile / portrait**: board on top, piece tray + controls below.
/// - **Tablet / landscape / desktop**: side-by-side board and controls panel.
///
/// Responsibilities:
/// - Registers the local player identity with [GameNotifier].
/// - Renders the board, piece tray, player panels, and turn indicator.
/// - Handles navigation to [ScoreScreen] when the game ends.
/// - Shows the connection status banner during reconnects.
class GameScreen extends ConsumerStatefulWidget {
  final String gameId;

  const GameScreen({super.key, required this.gameId});

  @override
  ConsumerState<GameScreen> createState() => _GameScreenState();
}

class _GameScreenState extends ConsumerState<GameScreen> {
  @override
  void initState() {
    super.initState();
    WidgetsBinding.instance.addPostFrameCallback((_) => _bootstrap());
  }

  /// Wires the local player identity to the [GameNotifier] and triggers the
  /// initial state fetch.
  void _bootstrap() {
    final lobby = ref.read(lobbyNotifierProvider);
    final notifier = ref.read(gameNotifierProvider(widget.gameId).notifier);

    if (lobby.localPlayerId.isNotEmpty && lobby.localColors.isNotEmpty) {
      notifier.setLocalIdentity(
        playerId: lobby.localPlayerId,
        localColors: lobby.localColors,
      );

      // Register every non-local slot as an AI seat so the backend's
      // advance_ai_turns() loop automatically plays their moves.
      // Compound lobby keys (blue_red, yellow_green) must be expanded to the
      // individual engine color names that the backend expects.
      final localId = lobby.localPlayerId;
      for (final entry in lobby.colorToPlayerId.entries) {
        if (entry.value == localId) continue;
        final engineColors = switch (entry.key) {
          'blue_red' => ['blue', 'red'],
          'yellow_green' => ['yellow', 'green'],
          _ => [entry.key],
        };
        for (final color in engineColors) {
          notifier.attachAi(entry.value, color);
        }
      }

      // Three-player: yellow and red players also rotate through the shared
      // green color, so they need to be registered as AI for green too.
      if (lobby.mode == GameModeOption.threePlayer) {
        for (final entry in lobby.colorToPlayerId.entries) {
          if (entry.value == localId) continue; // human – plays own green turns
          if (entry.key == 'green') continue; // not a real owned slot
          notifier.attachAi(entry.value, 'green');
        }
      }
    } else {
      // Spectator fallback (e.g. opened directly via deep link)
      notifier.subscribeAsSpectator();
    }
  }

  @override
  Widget build(BuildContext context) {
    final gameState = ref.watch(gameNotifierProvider(widget.gameId));
    final lobby = ref.read(lobbyNotifierProvider);

    // Auto-navigate to score screen on game finished.
    // Only fire on the transition TO finished so that a stale provider state
    // (from a previous visit to the same gameId) doesn't re-trigger navigation.
    ref.listen(gameNotifierProvider(widget.gameId), (prev, next) {
      if (next.gameState?.isFinished == true &&
          prev?.gameState?.isFinished != true) {
        context.go('$kRouteScore/${widget.gameId}');
      }
      // Show backend / engine errors as a SnackBar (e.g. pass rejected)
      if (next.errorMessage != null &&
          next.errorMessage != prev?.errorMessage) {
        ScaffoldMessenger.of(context).showSnackBar(
          SnackBar(
            content: Text(next.errorMessage!),
            backgroundColor: Theme.of(context).colorScheme.errorContainer,
          ),
        );
      }
    });

    return Scaffold(
      appBar: AppBar(
        title: _GameTitle(gameId: widget.gameId, gameState: gameState),
        centerTitle: false,
        actions: [
          _PassButton(gameId: widget.gameId),
          const Gap(4),
          IconButton(
            icon: const Icon(Icons.home_rounded),
            tooltip: 'Zur Lobby',
            onPressed: () => context.go(kRouteHome),
          ),
          const Gap(8),
        ],
      ),
      body: Column(
        children: [
          const ConnectionStatusBanner(),
          Expanded(
            child: _GameLayout(
              gameId: widget.gameId,
              lobby: lobby,
              gameState: gameState,
            ),
          ),
        ],
      ),
    );
  }
}

// ──────────────────────────────────────────────────────────────────────────────
// Layout
// ──────────────────────────────────────────────────────────────────────────────

/// Full-screen column layout: player strip → board → bottom tray.
///
/// The active player is communicated by the thick-bordered card in the player
/// strip above the board, so no separate turn banner is needed.
/// The game board fills all available vertical space between the player strip
/// and the bottom piece tray, giving it the maximum possible size on every
/// screen dimension.
class _GameLayout extends StatelessWidget {
  final String gameId;
  final LobbyState lobby;
  final GameNotifierState gameState;

  const _GameLayout({
    required this.gameId,
    required this.lobby,
    required this.gameState,
  });

  @override
  Widget build(BuildContext context) {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        // Player info strip – always visible above the board.
        Padding(
          padding: const EdgeInsets.fromLTRB(8, 8, 8, 4),
          child: _PlayerInfoRow(gameId: gameId),
        ),
        // No-moves hint sits directly below the player strip.
        _NoMovesHint(gameId: gameId),
        // Board takes all remaining vertical space.
        Expanded(
          child: Padding(
            padding: const EdgeInsets.fromLTRB(8, 4, 8, 4),
            child: GameBoard(gameId: gameId),
          ),
        ),
        // Piece tray + orientation selector pinned to the bottom.
        _BottomTray(gameId: gameId),
      ],
    );
  }
}

/// Bottom section of the game screen.
///
/// Structure (top to bottom):
/// 1. [PieceOrientationSelector] row – slides in with [AnimatedSize] only
///    when a piece is selected, invisible otherwise.
/// 2. Horizontally scrollable piece tray – always visible.
class _BottomTray extends StatelessWidget {
  final String gameId;

  const _BottomTray({required this.gameId});

  @override
  Widget build(BuildContext context) {
    final cs = Theme.of(context).colorScheme;
    return SafeArea(
      top: false,
      left: false,
      right: false,
      child: Container(
        decoration: BoxDecoration(
          color: cs.surfaceContainer,
          border: Border(top: BorderSide(color: cs.outlineVariant, width: 1)),
        ),
        child: Column(
          mainAxisSize: MainAxisSize.min,
          children: [
            // Orientation selector row – animates in when a piece is selected.
            AnimatedSize(
              duration: const Duration(milliseconds: 200),
              curve: Curves.easeInOut,
              child: const PieceOrientationSelector(),
            ),
            // Piece tray – always visible, needs a bounded height for ListView.
            SizedBox(height: 108, child: PieceTray(gameId: gameId)),
          ],
        ),
      ),
    );
  }
}

// ──────────────────────────────────────────────────────────────────────────────
// Sub-widgets
// ──────────────────────────────────────────────────────────────────────────────

/// Horizontal strip showing one card per player above the board.
///
/// Active player cards use a thick border. In three-player mode a player
/// whose turn it is to move the shared *green* colour gets an additional
/// green accent border on top of their primary colour border.
class _PlayerInfoRow extends ConsumerWidget {
  final String gameId;
  const _PlayerInfoRow({required this.gameId});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final gameState = ref.watch(gameNotifierProvider(gameId));
    final turnOrder = gameState.gameState?.turnOrder ?? [];
    final lobby = ref.read(lobbyNotifierProvider);

    final currentColor = gameState.gameState?.currentColor;
    final sharedIdx = gameState.gameState?.sharedColorTurnIndex;

    final visibleColors = [
      for (final color in turnOrder)
        if (!(lobby.mode == GameModeOption.threePlayer && color == 'green'))
          color,
    ];

    if (visibleColors.isEmpty) return const SizedBox.shrink();

    return Row(
      children: [
        for (int i = 0; i < visibleColors.length; i++) ...[
          if (i > 0) const SizedBox(width: 4),
          Expanded(
            child: () {
              final color = visibleColors[i];
              final playerId = lobby.colorToPlayerId[color] ?? color;
              final displayName =
                  lobby.playerNames[playerId] ??
                  playerId.substring(0, playerId.length.clamp(0, 8));

              // Determine whether this player card is active and, if so, which
              // colour they are currently playing (may differ from their own
              // in three-player mode when the shared green rotates to them).
              String? currentTurnColor;
              if (currentColor == color) {
                // Playing their own colour.
                currentTurnColor = color;
              } else if (currentColor == 'green' &&
                  lobby.mode == GameModeOption.threePlayer &&
                  sharedIdx != null &&
                  visibleColors.isNotEmpty) {
                // Green rotation: visibleColors[sharedIdx % n] is the owner.
                final ownerIndex = sharedIdx % visibleColors.length;
                if (ownerIndex == i) {
                  currentTurnColor = 'green';
                }
              }

              return PlayerInfoPanel(
                color: color,
                playerId: displayName,
                cellCount: gameState.gameState?.countForColor(color) ?? 0,
                isActive: currentTurnColor != null,
                currentTurnColor: currentTurnColor,
              );
            }(),
          ),
        ],
      ],
    );
  }
}

class _GameTitle extends ConsumerWidget {
  final String gameId;
  final GameNotifierState gameState;

  const _GameTitle({required this.gameId, required this.gameState});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final lobby = ref.read(lobbyNotifierProvider);
    final mode = gameState.gameState?.mode;
    final modePart = mode != null ? _modeLabel(mode) : null;

    final String title;
    if (modePart != null && lobby.gameName.isNotEmpty) {
      title = '$modePart – ${lobby.gameName}';
    } else if (lobby.gameName.isNotEmpty) {
      title = lobby.gameName;
    } else if (modePart != null) {
      title = '$modePart · ${gameId.substring(0, 8)}';
    } else {
      title = gameId.substring(0, 8);
    }
    return Text(title);
  }

  String _modeLabel(String mode) => switch (mode) {
    'duo' => 'Duo',
    'two_player' => '2-Player',
    'three_player' => '3-Player',
    'four_player' => '4-Player',
    _ => mode,
  };
}

/// Shows a warning banner when it is the local player's turn but there are
/// no legal moves left — prompting them to press Pass.
class _NoMovesHint extends ConsumerWidget {
  final String gameId;
  const _NoMovesHint({required this.gameId});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final gs = ref.watch(gameNotifierProvider(gameId));
    final lobby = ref.read(lobbyNotifierProvider);

    final isMyTurn =
        gs.gameState != null &&
        lobby.localColors.contains(gs.gameState!.currentColor);
    final noMoves = isMyTurn && gs.legalMoves.isEmpty && !gs.isLoadingMove;

    if (!noMoves) return const SizedBox.shrink();

    final cs = Theme.of(context).colorScheme;
    return Padding(
      padding: const EdgeInsets.fromLTRB(8, 4, 8, 0),
      child: Container(
        padding: const EdgeInsets.symmetric(horizontal: 10, vertical: 6),
        decoration: BoxDecoration(
          color: cs.errorContainer,
          borderRadius: BorderRadius.circular(8),
        ),
        child: Row(
          children: [
            Icon(Icons.block_rounded, size: 14, color: cs.onErrorContainer),
            const Gap(6),
            Expanded(
              child: Text(
                'Kein Zug möglich – bitte passen!',
                style: Theme.of(context).textTheme.labelSmall?.copyWith(
                  color: cs.onErrorContainer,
                  fontWeight: FontWeight.bold,
                ),
              ),
            ),
          ],
        ),
      ),
    );
  }
}

class _PassButton extends ConsumerWidget {
  final String gameId;
  const _PassButton({required this.gameId});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final gameState = ref.watch(gameNotifierProvider(gameId));
    final lobby = ref.read(lobbyNotifierProvider);

    final isMyTurn =
        gameState.gameState != null &&
        lobby.localColors.contains(gameState.gameState!.currentColor);
    // legalMoves is empty either because genuinely no moves exist, or because
    // the backend hasn't responded yet. In both cases the player should be
    // able to press Pass (the engine will reject it if moves are still possible).
    final noMoves = gameState.legalMoves.isEmpty && !gameState.isLoadingMove;

    if (!isMyTurn) return const SizedBox.shrink();

    final cs = Theme.of(context).colorScheme;
    // When noMoves: prominent red + shimmer to guide the user.
    // When moves exist: muted style so the player knows pass is a last resort.
    return TextButton.icon(
      onPressed:
          gameState.isLoadingMove
              ? null
              : () =>
                  ref.read(gameNotifierProvider(gameId).notifier).passMove(),
      icon: const Icon(Icons.skip_next_rounded),
      label: const Text('Pass'),
      style: TextButton.styleFrom(
        foregroundColor: noMoves ? cs.error : cs.onSurfaceVariant,
      ),
    ).animate(target: noMoves ? 1 : 0).shimmer(duration: 1200.ms);
  }
}
