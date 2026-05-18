import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_animate/flutter_animate.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:gap/gap.dart';
import 'package:go_router/go_router.dart';
import 'package:uuid/uuid.dart';

import '../../core/constants.dart';
import '../../core/router.dart';
import '../../data/websocket_service.dart';
import '../../domain/lobby_notifier.dart';
import '../../domain/providers.dart';

/// Welcome screen: the very first screen the user sees.
///
/// Responsibilities:
/// - Let the user configure the server URL and their player name (persisted).
/// - Choose a game mode (Duo / Classic 2/3/4-Player) and scoring variant.
/// - Configure all player slots (opponent names / AI).
/// - Create a new game **or** join an existing one by entering a game_id.
///
/// On success the user is navigated to [GameScreen].
class HomeScreen extends ConsumerStatefulWidget {
  const HomeScreen({super.key});

  @override
  ConsumerState<HomeScreen> createState() => _HomeScreenState();
}

class _HomeScreenState extends ConsumerState<HomeScreen>
    with SingleTickerProviderStateMixin {
  late TabController _tabController;

  // Shared fields
  final _serverUrlCtrl = TextEditingController(text: 'ws://localhost:8000/ws');
  final _playerNameCtrl = TextEditingController();
  final _gameNameCtrl = TextEditingController();

  // Create-game fields
  final _p2Ctrl = TextEditingController(); // opponent 2
  final _p3Ctrl = TextEditingController(); // opponent 3
  final _p4Ctrl = TextEditingController(); // opponent 4
  final _firstColorCtrl = ValueNotifier<String>('blue');

  // Join-game field
  final _joinGameIdCtrl = TextEditingController();

  final _createFormKey = GlobalKey<FormState>();
  final _joinFormKey = GlobalKey<FormState>();

  @override
  void initState() {
    super.initState();
    _tabController = TabController(length: 2, vsync: this);
    // Pre-fill persisted player name
    WidgetsBinding.instance.addPostFrameCallback((_) {
      final prefs = ref.read(preferencesServiceProvider);
      _playerNameCtrl.text = prefs.playerName;
      _serverUrlCtrl.text = prefs.serverUrl;
    });
  }

  @override
  void dispose() {
    _tabController.dispose();
    _serverUrlCtrl.dispose();
    _playerNameCtrl.dispose();
    _gameNameCtrl.dispose();
    _p2Ctrl.dispose();
    _p3Ctrl.dispose();
    _p4Ctrl.dispose();
    _firstColorCtrl.dispose();
    _joinGameIdCtrl.dispose();
    super.dispose();
  }

  // ── Lifecycle ───────────────────────────────────────────────────────────────

  @override
  Widget build(BuildContext context) {
    final lobby = ref.watch(lobbyNotifierProvider);

    // Navigate to game screen once lobby is ready
    ref.listen(lobbyNotifierProvider, (prev, next) {
      if (next.phase == LobbyPhase.ready && next.gameId.isNotEmpty) {
        context.go('$kRouteGame/${next.gameId}');
      }
    });

    return Scaffold(
      body: SafeArea(
        child: Center(
          child: ConstrainedBox(
            constraints: const BoxConstraints(maxWidth: 520),
            child: SingleChildScrollView(
              padding: const EdgeInsets.all(24),
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.stretch,
                children: [
                  _Header(),
                  const Gap(24),
                  _ServerRow(controller: _serverUrlCtrl),
                  const Gap(12),
                  _PlayerNameField(controller: _playerNameCtrl),
                  const Gap(20),
                  _ModeAndScoringRow(),
                  const Gap(20),
                  TabBar(
                    controller: _tabController,
                    tabs: const [
                      Tab(text: 'Create Game'),
                      Tab(text: 'Join Game'),
                    ],
                  ),
                  const Gap(16),
                  AnimatedBuilder(
                    animation: _tabController,
                    builder: (context, _) {
                      if (_tabController.index == 0) {
                        return _CreateGameTab(
                          formKey: _createFormKey,
                          gameNameCtrl: _gameNameCtrl,
                          p2Ctrl: _p2Ctrl,
                          p3Ctrl: _p3Ctrl,
                          p4Ctrl: _p4Ctrl,
                          firstColorNotifier: _firstColorCtrl,
                          onCreate: _onCreateGame,
                        );
                      }
                      return _JoinGameTab(
                        formKey: _joinFormKey,
                        gameIdCtrl: _joinGameIdCtrl,
                        onJoin: _onJoinGame,
                      );
                    },
                  ),
                  if (lobby.errorMessage != null) ...[
                    const Gap(12),
                    _ErrorBanner(message: lobby.errorMessage!),
                  ],
                ],
              ),
            ),
          ),
        ),
      ),
    );
  }

  // ── Handlers ────────────────────────────────────────────────────────────────

  Future<void> _onCreateGame() async {
    if (!_createFormKey.currentState!.validate()) return;

    final lobbyNotifier = ref.read(lobbyNotifierProvider.notifier);
    final lobby = ref.read(lobbyNotifierProvider);
    final mode = lobby.mode;
    final displayName = _playerNameCtrl.text.trim();
    const uuid = Uuid();
    // Engine requires UUID strings as player IDs.
    final localUuid = uuid.v4();
    final gameId = uuid.v4();

    // Persist display name + server url
    try {
      await ref
          .read(preferencesServiceProvider)
          .setServerUrl(_serverUrlCtrl.text.trim());
      await ref
          .read(preferencesServiceProvider)
          .setPlayerName(displayName);
    } catch (_) {}

    lobbyNotifier.setGameName(_gameNameCtrl.text.trim());
    lobbyNotifier.setLocalPlayerId(localUuid);

    // Connect to server
    await lobbyNotifier.connectToServer(_serverUrlCtrl.text.trim());

    // Build colour→UUID map. Each non-local slot gets its own fresh UUID
    // (the engine refuses duplicate or non-UUID player IDs).
    final ai1 = uuid.v4();
    final ai2 = uuid.v4();
    final ai3 = uuid.v4();

    final Map<String, String> slots = switch (mode) {
      GameModeOption.duo => {
          'black': localUuid,
          'white': ai1,
        },
      GameModeOption.twoPlayer => {
          'blue_red': localUuid,
          'yellow_green': ai1,
        },
      GameModeOption.threePlayer => {
          'blue': localUuid,
          'yellow': ai1,
          'red': ai2,
          'green': localUuid, // shared green defaults to local player
        },
      GameModeOption.fourPlayer => {
          'blue': localUuid,
          'yellow': ai1,
          'red': ai2,
          'green': ai3,
        },
    };

    // Build display-name map: local player by name, others as "AI Player N"
    final Map<String, String> playerNames = {
      localUuid: displayName.isNotEmpty ? displayName : 'You',
      ai1: 'AI Player 1',
      ai2: 'AI Player 2',
      ai3: 'AI Player 3',
    };
    lobbyNotifier.setPlayerNames(playerNames);

    lobbyNotifier.createGame(
      gameId: gameId,
      colorToPlayerId: slots,
      firstColor: _firstColorCtrl.value,
    );
  }

  Future<void> _onJoinGame() async {
    if (!_joinFormKey.currentState!.validate()) return;

    final localId = _playerNameCtrl.text.trim();
    final gameId = _joinGameIdCtrl.text.trim();

    try {
      await ref
          .read(preferencesServiceProvider)
          .setServerUrl(_serverUrlCtrl.text.trim());
      await ref.read(preferencesServiceProvider).setPlayerName(localId);
    } catch (_) {}

    final lobbyNotifier = ref.read(lobbyNotifierProvider.notifier);
    lobbyNotifier.setLocalPlayerId(localId);
    await lobbyNotifier.connectToServer(_serverUrlCtrl.text.trim());
    lobbyNotifier.joinGame(gameId);
  }
}

// ──────────────────────────────────────────────────────────────────────────────
// Sub-widgets
// ──────────────────────────────────────────────────────────────────────────────

class _Header extends StatelessWidget {
  @override
  Widget build(BuildContext context) {
    final cs = Theme.of(context).colorScheme;
    return Column(
      children: [
        Icon(Icons.grid_on_rounded, size: 56, color: cs.primary)
            .animate()
            .fadeIn(duration: 600.ms)
            .scale(begin: const Offset(0.7, 0.7)),
        const Gap(8),
        Text(
          'Blocus',
          style: Theme.of(context)
              .textTheme
              .displaySmall
              ?.copyWith(fontWeight: FontWeight.bold, color: cs.primary),
        ).animate().fadeIn(delay: 200.ms),
        Text(
          'Blokus Classic & Duo',
          style: Theme.of(context)
              .textTheme
              .bodyMedium
              ?.copyWith(color: cs.onSurfaceVariant),
        ).animate().fadeIn(delay: 400.ms),
      ],
    );
  }
}

class _ServerRow extends ConsumerWidget {
  final TextEditingController controller;
  const _ServerRow({required this.controller});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final connState = ref.watch(connectionStateProvider);
    final isConnected = connState.valueOrNull == WsConnectionState.connected;
    return Row(
      children: [
        Expanded(
          child: TextFormField(
            controller: controller,
            decoration: InputDecoration(
              labelText: 'Server URL',
              hintText: 'ws://localhost:8000/ws',
              prefixIcon: const Icon(Icons.dns_rounded),
              suffixIcon: isConnected
                  ? const Icon(Icons.check_circle, color: Colors.green)
                  : null,
              border: const OutlineInputBorder(),
            ),
            keyboardType: TextInputType.url,
            autocorrect: false,
          ),
        ),
      ],
    );
  }
}

class _PlayerNameField extends StatelessWidget {
  final TextEditingController controller;
  const _PlayerNameField({required this.controller});

  @override
  Widget build(BuildContext context) {
    return TextFormField(
      controller: controller,
      decoration: const InputDecoration(
        labelText: 'Your Player Name',
        hintText: 'e.g. alice',
        prefixIcon: Icon(Icons.person_rounded),
        border: OutlineInputBorder(),
      ),
      inputFormatters: [
        FilteringTextInputFormatter.deny(RegExp(r'\s')),
        LengthLimitingTextInputFormatter(32),
      ],
    );
  }
}

class _ModeAndScoringRow extends ConsumerWidget {
  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final lobby = ref.watch(lobbyNotifierProvider);
    final notifier = ref.read(lobbyNotifierProvider.notifier);

    return Row(
      children: [
        Expanded(
          flex: 3,
          child: DropdownButtonFormField<GameModeOption>(
            initialValue: lobby.mode,
            decoration: const InputDecoration(
              labelText: 'Mode',
              border: OutlineInputBorder(),
            ),
            items: GameModeOption.values
                .map((m) =>
                    DropdownMenuItem(value: m, child: Text(m.displayName)))
                .toList(),
            onChanged: (m) => m != null ? notifier.setMode(m) : null,
          ),
        ),
        const Gap(12),
        Expanded(
          flex: 2,
          child: DropdownButtonFormField<String>(
            initialValue: lobby.scoring,
            decoration: const InputDecoration(
              labelText: 'Scoring',
              border: OutlineInputBorder(),
            ),
            items: [
              const DropdownMenuItem(value: 'basic', child: Text('Basic')),
              const DropdownMenuItem(
                  value: 'advanced', child: Text('Advanced')),
            ],
            onChanged: lobby.mode == GameModeOption.duo
                ? null // Duo is always advanced
                : (s) => s != null ? notifier.setScoring(s) : null,
          ),
        ),
      ],
    );
  }
}

// ── Create-Game tab ──────────────────────────────────────────────────────────

class _CreateGameTab extends ConsumerWidget {
  final GlobalKey<FormState> formKey;
  final TextEditingController gameNameCtrl;
  final TextEditingController p2Ctrl, p3Ctrl, p4Ctrl;
  final ValueNotifier<String> firstColorNotifier;
  final VoidCallback onCreate;

  const _CreateGameTab({
    required this.formKey,
    required this.gameNameCtrl,
    required this.p2Ctrl,
    required this.p3Ctrl,
    required this.p4Ctrl,
    required this.firstColorNotifier,
    required this.onCreate,
  });

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final lobby = ref.watch(lobbyNotifierProvider);
    final mode = lobby.mode;

    return Form(
      key: formKey,
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
            // Game name
            TextFormField(
              controller: gameNameCtrl,
              decoration: const InputDecoration(
                labelText: 'Game Name',
                hintText: 'e.g. Friday Night Blokus',
                prefixIcon: Icon(Icons.sports_esports_rounded),
                border: OutlineInputBorder(),
              ),
              inputFormatters: [
                LengthLimitingTextInputFormatter(48),
              ],
            ),
            const Gap(10),
            // Opponent-slot fields (vary by mode)
            _opponentField(
              context,
              ctrl: p2Ctrl,
              label: _opponentLabel(mode, 2),
            ),
            if (mode == GameModeOption.threePlayer ||
                mode == GameModeOption.fourPlayer) ...[
              const Gap(10),
              _opponentField(context, ctrl: p3Ctrl, label: 'Player 3 (Red)'),
            ],
            if (mode == GameModeOption.fourPlayer) ...[
              const Gap(10),
              _opponentField(context, ctrl: p4Ctrl, label: 'Player 4 (Green)'),
            ],
            const Gap(16),
            // First-colour picker (only for Duo and 4-Player)
            if (mode == GameModeOption.duo ||
                mode == GameModeOption.fourPlayer) ...[
              ValueListenableBuilder<String>(
                valueListenable: firstColorNotifier,
                builder: (_, val, _) => DropdownButtonFormField<String>(
                  initialValue: val,
                  decoration: const InputDecoration(
                    labelText: 'First colour',
                    border: OutlineInputBorder(),
                  ),
                  items: (mode == GameModeOption.duo
                          ? kDuoColors
                          : kClassicColors)
                      .map((c) => DropdownMenuItem(
                          value: c,
                          child: Row(
                            children: [
                              CircleAvatar(
                                  radius: 8,
                                  backgroundColor: colorForPlayer(c)),
                              const Gap(8),
                              Text(c),
                            ],
                          )))
                      .toList(),
                  onChanged: (c) => c != null ? firstColorNotifier.value = c : null,
                ),
              ),
              const Gap(16),
            ],
            FilledButton.icon(
              onPressed: lobby.isLoading ? null : onCreate,
              icon: lobby.isLoading
                  ? const SizedBox(
                      width: 16,
                      height: 16,
                      child: CircularProgressIndicator(strokeWidth: 2),
                    )
                  : const Icon(Icons.add_circle_rounded),
              label: const Text('Create Game'),
            ),
          ],
        ),
    );
  }

  String _opponentLabel(GameModeOption mode, int slot) => switch (mode) {
        GameModeOption.duo => 'Opponent (White) — leave blank for AI',
        GameModeOption.twoPlayer =>
          'Opponent (Yellow/Green) — leave blank for AI',
        _ => 'Player $slot — leave blank for AI',
      };

  Widget _opponentField(
    BuildContext context, {
    required TextEditingController ctrl,
    required String label,
  }) =>
      TextFormField(
        controller: ctrl,
        decoration: InputDecoration(
          labelText: label,
          prefixIcon: const Icon(Icons.person_outline_rounded),
          border: const OutlineInputBorder(),
        ),
        inputFormatters: [
          FilteringTextInputFormatter.deny(RegExp(r'\s')),
          LengthLimitingTextInputFormatter(32),
        ],
      );
}

// ── Join-Game tab ────────────────────────────────────────────────────────────

class _JoinGameTab extends ConsumerWidget {
  final GlobalKey<FormState> formKey;
  final TextEditingController gameIdCtrl;
  final VoidCallback onJoin;

  const _JoinGameTab({
    required this.formKey,
    required this.gameIdCtrl,
    required this.onJoin,
  });

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final lobby = ref.watch(lobbyNotifierProvider);

    return Form(
      key: formKey,
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          TextFormField(
            controller: gameIdCtrl,
            decoration: const InputDecoration(
              labelText: 'Game ID',
              hintText: 'Paste or type the game ID shared by the host',
              prefixIcon: Icon(Icons.tag_rounded),
              border: OutlineInputBorder(),
            ),
            validator: (v) =>
                (v == null || v.trim().isEmpty) ? 'Game ID is required' : null,
          ),
          const Gap(16),
          FilledButton.icon(
            onPressed: lobby.isLoading ? null : onJoin,
            icon: lobby.isLoading
                ? const SizedBox(
                    width: 16,
                    height: 16,
                    child: CircularProgressIndicator(strokeWidth: 2),
                  )
                : const Icon(Icons.login_rounded),
            label: const Text('Join Game'),
          ),
        ],
      ),
    );
  }
}

class _ErrorBanner extends StatelessWidget {
  final String message;
  const _ErrorBanner({required this.message});

  @override
  Widget build(BuildContext context) {
    return Container(
      padding: const EdgeInsets.all(12),
      decoration: BoxDecoration(
        color: Theme.of(context).colorScheme.errorContainer,
        borderRadius: BorderRadius.circular(8),
      ),
      child: Row(
        children: [
          Icon(Icons.error_outline,
              color: Theme.of(context).colorScheme.onErrorContainer),
          const Gap(8),
          Expanded(
            child: Text(
              message,
              style: TextStyle(
                  color: Theme.of(context).colorScheme.onErrorContainer),
            ),
          ),
        ],
      ),
    ).animate().shake(hz: 2, duration: 400.ms);
  }
}
