import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_animate/flutter_animate.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:gap/gap.dart';
import 'package:go_router/go_router.dart';

import '../../core/constants.dart';
import '../../core/router.dart';
import '../../data/models/saved_game.dart';
import '../../data/websocket_service.dart';
import '../../domain/lobby_notifier.dart';
import '../../domain/providers.dart';

/// Welcome screen: the very first screen the user sees.
///
/// Responsibilities:
/// - Let the user configure the server URL and their player name (persisted).
/// - **Create Game tab**: choose mode / scoring / name and create a local game
///   entry (no backend call yet).
/// - **Join Game tab**: pick a previously created game, assign player slots,
///   and press "Start Game" to connect to the backend and begin play.
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

  // ── Shared fields ────────────────────────────────────────────────────────────
  final _serverUrlCtrl = TextEditingController(text: 'ws://localhost:8000/ws');
  final _playerNameCtrl = TextEditingController();

  // ── Create Game fields ───────────────────────────────────────────────────────
  final _gameNameCtrl = TextEditingController();
  final _createFormKey = GlobalKey<FormState>();

  // ── Join Game state ──────────────────────────────────────────────────────────
  List<SavedGame> _savedGames = [];
  SavedGame? _selectedGame;
  List<TextEditingController> _slotControllers = [];

  @override
  void initState() {
    super.initState();
    _tabController = TabController(length: 2, vsync: this);
    _tabController.addListener(_onTabChanged);
    WidgetsBinding.instance.addPostFrameCallback((_) {
      final prefs = ref.read(preferencesServiceProvider);
      _playerNameCtrl.text = prefs.playerName;
      _serverUrlCtrl.text = prefs.serverUrl;
      setState(() => _savedGames = prefs.savedGames);
    });
  }

  @override
  void dispose() {
    _tabController
      ..removeListener(_onTabChanged)
      ..dispose();
    _serverUrlCtrl.dispose();
    _playerNameCtrl.dispose();
    _gameNameCtrl.dispose();
    for (final c in _slotControllers) {
      c.dispose();
    }
    super.dispose();
  }

  void _onTabChanged() {
    if (!_tabController.indexIsChanging && _tabController.index == 1) {
      // Refresh saved-games list whenever the Join tab becomes active.
      final fresh = ref.read(preferencesServiceProvider).savedGames;
      setState(() {
        _savedGames = fresh;
        // Keep selected game reference in sync with refreshed list.
        if (_selectedGame != null) {
          final match = fresh.where((g) => g.gameId == _selectedGame!.gameId);
          if (match.isEmpty) {
            _selectedGame = null;
            for (final c in _slotControllers) {
              c.dispose();
            }
            _slotControllers = [];
          }
        }
      });
    }
  }

  void _onGameSelected(SavedGame? game) {
    for (final c in _slotControllers) {
      c.dispose();
    }
    if (game == null) {
      setState(() {
        _selectedGame = null;
        _slotControllers = [];
      });
      return;
    }

    final slots = game.slots;
    final playerName = _playerNameCtrl.text.trim();

    // If the current player is already the creator (slot 0), don't pre-fill
    // their name into any other slot — they don't need a second entry.
    final creatorName = game.players[slots[0]] ?? '';
    final playerIsCreator = playerName.isNotEmpty && playerName == creatorName;
    bool localPlaced = false;

    final controllers = List.generate(slots.length, (i) {
      final slot = slots[i];
      final existing = game.players[slot] ?? '';
      final String initial;
      if (i == 0) {
        // First slot = creator (read-only, always from SavedGame).
        initial = existing;
      } else if (existing.isNotEmpty) {
        initial = existing;
      } else if (!localPlaced && !playerIsCreator) {
        // Pre-fill the first free slot with the current player's name,
        // but only when this player is not already the creator.
        initial = playerName;
        localPlaced = true;
      } else {
        initial = '';
      }
      return TextEditingController(text: initial);
    });

    setState(() {
      _selectedGame = game;
      _slotControllers = controllers;
    });
  }

  // ── Handlers ─────────────────────────────────────────────────────────────────

  Future<void> _createLocalGame() async {
    if (!_createFormKey.currentState!.validate()) return;

    final prefs = ref.read(preferencesServiceProvider);
    final playerName = _playerNameCtrl.text.trim();

    try {
      await prefs.setServerUrl(_serverUrlCtrl.text.trim());
      await prefs.setPlayerName(playerName);
    } catch (_) {}

    ref
        .read(lobbyNotifierProvider.notifier)
        .createLocalGame(
          creatorName: playerName,
          gameName: _gameNameCtrl.text.trim(),
        );

    if (mounted) {
      ScaffoldMessenger.of(context).showSnackBar(
        const SnackBar(
          content: Text('Game created! Switch to "Join Game" to start it.'),
          duration: Duration(seconds: 3),
        ),
      );
      setState(() {
        _savedGames = ref.read(preferencesServiceProvider).savedGames;
        _gameNameCtrl.clear();
      });
    }
  }

  Future<void> _startGame() async {
    if (_selectedGame == null) return;

    final playerName = _playerNameCtrl.text.trim();
    final slots = _selectedGame!.slots;

    final Map<String, String> finalSlotNames = {
      for (int i = 0; i < slots.length && i < _slotControllers.length; i++)
        slots[i]: _slotControllers[i].text.trim(),
    };

    try {
      await ref
          .read(preferencesServiceProvider)
          .setServerUrl(_serverUrlCtrl.text.trim());
      await ref.read(preferencesServiceProvider).setPlayerName(playerName);
    } catch (_) {}

    await ref
        .read(lobbyNotifierProvider.notifier)
        .startGame(
          savedGame: _selectedGame!,
          localPlayerName: playerName,
          finalSlotNames: finalSlotNames,
          serverUrl: _serverUrlCtrl.text.trim(),
        );
  }

  // ── Build ─────────────────────────────────────────────────────────────────────

  @override
  Widget build(BuildContext context) {
    final lobby = ref.watch(lobbyNotifierProvider);

    ref.listen(lobbyNotifierProvider, (prev, next) {
      if (next.phase == LobbyPhase.ready &&
          prev?.phase != LobbyPhase.ready &&
          next.gameId.isNotEmpty) {
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
                          onCreate: _createLocalGame,
                        );
                      }
                      return _JoinGameTab(
                        savedGames: _savedGames,
                        selectedGame: _selectedGame,
                        slotControllers: _slotControllers,
                        onGameSelected: _onGameSelected,
                        onStart: _startGame,
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
}

// ──────────────────────────────────────────────────────────────────────────────
// Shared sub-widgets
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
          style: Theme.of(context).textTheme.displaySmall?.copyWith(
            fontWeight: FontWeight.bold,
            color: cs.primary,
          ),
        ).animate().fadeIn(delay: 200.ms),
        Text(
          'Blokus Classic & Duo',
          style: Theme.of(
            context,
          ).textTheme.bodyMedium?.copyWith(color: cs.onSurfaceVariant),
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
    return TextFormField(
      controller: controller,
      decoration: InputDecoration(
        labelText: 'Server URL',
        hintText: 'ws://localhost:8000/ws',
        prefixIcon: const Icon(Icons.dns_rounded),
        suffixIcon:
            isConnected
                ? const Icon(Icons.check_circle, color: Colors.green)
                : null,
        border: const OutlineInputBorder(),
      ),
      keyboardType: TextInputType.url,
      autocorrect: false,
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
          Icon(
            Icons.error_outline,
            color: Theme.of(context).colorScheme.onErrorContainer,
          ),
          const Gap(8),
          Expanded(
            child: Text(
              message,
              style: TextStyle(
                color: Theme.of(context).colorScheme.onErrorContainer,
              ),
            ),
          ),
        ],
      ),
    ).animate().shake(hz: 2, duration: 400.ms);
  }
}

// ──────────────────────────────────────────────────────────────────────────────
// Create Game tab
// ──────────────────────────────────────────────────────────────────────────────

class _CreateGameTab extends ConsumerWidget {
  final GlobalKey<FormState> formKey;
  final TextEditingController gameNameCtrl;
  final VoidCallback onCreate;

  const _CreateGameTab({
    required this.formKey,
    required this.gameNameCtrl,
    required this.onCreate,
  });

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final lobby = ref.watch(lobbyNotifierProvider);
    final notifier = ref.read(lobbyNotifierProvider.notifier);

    return Form(
      key: formKey,
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          // Mode
          DropdownButtonFormField<GameModeOption>(
            isExpanded: true,
            initialValue: lobby.mode,
            decoration: const InputDecoration(
              labelText: 'Mode',
              border: OutlineInputBorder(),
            ),
            items:
                GameModeOption.values
                    .map(
                      (m) => DropdownMenuItem(
                        value: m,
                        child: Text(m.displayName),
                      ),
                    )
                    .toList(),
            onChanged: (m) => m != null ? notifier.setMode(m) : null,
          ),
          const Gap(10),
          // Scoring
          DropdownButtonFormField<String>(
            isExpanded: true,
            initialValue: lobby.scoring,
            decoration: const InputDecoration(
              labelText: 'Scoring',
              border: OutlineInputBorder(),
            ),
            items: const [
              DropdownMenuItem(value: 'basic', child: Text('Basic')),
              DropdownMenuItem(value: 'advanced', child: Text('Advanced')),
            ],
            onChanged:
                lobby.mode == GameModeOption.duo
                    ? null // Duo is always advanced
                    : (s) => s != null ? notifier.setScoring(s) : null,
          ),
          const Gap(10),
          // Game name
          TextFormField(
            controller: gameNameCtrl,
            decoration: const InputDecoration(
              labelText: 'Game Name',
              hintText: 'e.g. Friday Night Blokus',
              prefixIcon: Icon(Icons.sports_esports_rounded),
              border: OutlineInputBorder(),
            ),
            inputFormatters: [LengthLimitingTextInputFormatter(48)],
          ),
          const Gap(16),
          FilledButton.icon(
            onPressed: onCreate,
            icon: const Icon(Icons.add_circle_rounded),
            label: const Text('Create Game'),
          ),
        ],
      ),
    );
  }
}

// ──────────────────────────────────────────────────────────────────────────────
// Join Game tab
// ──────────────────────────────────────────────────────────────────────────────

class _JoinGameTab extends ConsumerWidget {
  final List<SavedGame> savedGames;
  final SavedGame? selectedGame;
  final List<TextEditingController> slotControllers;
  final ValueChanged<SavedGame?> onGameSelected;
  final VoidCallback onStart;

  const _JoinGameTab({
    required this.savedGames,
    required this.selectedGame,
    required this.slotControllers,
    required this.onGameSelected,
    required this.onStart,
  });

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final lobby = ref.watch(lobbyNotifierProvider);

    return Column(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        // Game selector
        DropdownButtonFormField<SavedGame>(
          initialValue: selectedGame,
          isExpanded: true,
          decoration: const InputDecoration(
            labelText: 'Select Game',
            prefixIcon: Icon(Icons.sports_esports_rounded),
            border: OutlineInputBorder(),
          ),
          hint:
              savedGames.isEmpty
                  ? const Text('No games yet – create one first')
                  : const Text('Choose a game to join'),
          items:
              savedGames
                  .map(
                    (g) => DropdownMenuItem(
                      value: g,
                      child: Text(
                        g.displayLabel,
                        overflow: TextOverflow.ellipsis,
                      ),
                    ),
                  )
                  .toList(),
          onChanged: onGameSelected,
        ),

        // Details + slot rows (only when a game is selected)
        if (selectedGame != null) ...[
          const Gap(12),
          // Read-only game info
          Row(
            children: [
              Expanded(
                child: _ReadOnlyField(
                  label: 'Mode',
                  value: selectedGame!.modeDisplayName,
                ),
              ),
              const Gap(8),
              Expanded(
                child: _ReadOnlyField(
                  label: 'Scoring',
                  value:
                      selectedGame!.scoring == 'advanced'
                          ? 'Advanced'
                          : 'Basic',
                ),
              ),
            ],
          ),
          const Gap(12),
          // Player slot rows
          ...List.generate(selectedGame!.slots.length, (i) {
            return Padding(
              padding: const EdgeInsets.only(bottom: 8),
              child: _SlotRow(
                slotKey: selectedGame!.slots[i],
                controller: slotControllers[i],
                isReadOnly: i == 0, // creator slot is read-only
              ),
            );
          }),
        ],

        const Gap(16),
        FilledButton.icon(
          onPressed: (lobby.isLoading || selectedGame == null) ? null : onStart,
          icon:
              lobby.isLoading
                  ? const SizedBox(
                    width: 16,
                    height: 16,
                    child: CircularProgressIndicator(strokeWidth: 2),
                  )
                  : const Icon(Icons.play_arrow_rounded),
          label: const Text('Start Game'),
        ),
      ],
    );
  }
}

// ── Shared helper widgets ────────────────────────────────────────────────────

class _ReadOnlyField extends StatelessWidget {
  final String label;
  final String value;
  const _ReadOnlyField({required this.label, required this.value});

  @override
  Widget build(BuildContext context) {
    return InputDecorator(
      decoration: InputDecoration(
        labelText: label,
        border: const OutlineInputBorder(),
        filled: true,
      ),
      child: Text(value, style: Theme.of(context).textTheme.bodyMedium),
    );
  }
}

class _SlotRow extends StatelessWidget {
  final String slotKey;
  final TextEditingController controller;
  final bool isReadOnly;

  const _SlotRow({
    required this.slotKey,
    required this.controller,
    this.isReadOnly = false,
  });

  @override
  Widget build(BuildContext context) {
    // For compound slots like 'blue_red' use the first colour for the badge.
    final primaryColor = slotKey.split('_').first;
    final color = colorForPlayer(primaryColor);

    final label = switch (slotKey) {
      'black' => 'Black',
      'white' => 'White',
      'blue_red' => 'Blue / Red',
      'yellow_green' => 'Yellow / Green',
      'blue' => 'Blue',
      'yellow' => 'Yellow',
      'red' => 'Red',
      'green' => 'Green',
      _ => slotKey,
    };

    return TextFormField(
      controller: controller,
      readOnly: isReadOnly,
      decoration: InputDecoration(
        labelText:
            '$label${isReadOnly ? '  (host)' : '  – leave blank for AI'}',
        prefixIcon: Padding(
          padding: const EdgeInsets.all(12),
          child: CircleAvatar(radius: 8, backgroundColor: color),
        ),
        border: const OutlineInputBorder(),
        filled: isReadOnly,
      ),
      inputFormatters: [
        FilteringTextInputFormatter.deny(RegExp(r'\s')),
        LengthLimitingTextInputFormatter(32),
      ],
    );
  }
}
