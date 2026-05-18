import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:go_router/go_router.dart';
import '../presentation/screens/home_screen.dart';
import '../presentation/screens/game_screen.dart';
import '../presentation/screens/score_screen.dart';

// ──────────────────────────────────────────────────────────────────────────────
// Route name constants
// ──────────────────────────────────────────────────────────────────────────────

/// Named route for the welcome / game-setup screen.
const String kRouteHome = '/';

/// Named route for the main game screen.  Path param: `:gameId`.
const String kRouteGame = '/game';

/// Named route for the score / end-game screen.  Path param: `:gameId`.
const String kRouteScore = '/score';

// ──────────────────────────────────────────────────────────────────────────────
// Router provider
// ──────────────────────────────────────────────────────────────────────────────

/// Provides the singleton [GoRouter] instance to the widget tree.
///
/// The router is kept in a [Provider] so that it can be watched by [BlocusApp]
/// without being recreated on every rebuild.
final routerProvider = Provider<GoRouter>((ref) {
  return GoRouter(
    initialLocation: kRouteHome,
    debugLogDiagnostics: false,
    routes: [
      GoRoute(
        path: kRouteHome,
        name: 'home',
        builder: (context, state) => const HomeScreen(),
      ),
      GoRoute(
        path: '$kRouteGame/:gameId',
        name: 'game',
        builder: (context, state) {
          final gameId = state.pathParameters['gameId']!;
          return GameScreen(gameId: gameId);
        },
      ),
      GoRoute(
        path: '$kRouteScore/:gameId',
        name: 'score',
        builder: (context, state) {
          final gameId = state.pathParameters['gameId']!;
          return ScoreScreen(gameId: gameId);
        },
      ),
    ],
    // Global error page so the app never shows a blank screen on bad routes.
    errorBuilder:
        (context, state) => Scaffold(
          body: Center(
            child: Column(
              mainAxisSize: MainAxisSize.min,
              children: [
                const Icon(Icons.error_outline, size: 48),
                const SizedBox(height: 16),
                Text('Page not found: ${state.uri}'),
                const SizedBox(height: 16),
                FilledButton(
                  onPressed: () => context.go(kRouteHome),
                  child: const Text('Back to Home'),
                ),
              ],
            ),
          ),
        ),
  );
});
