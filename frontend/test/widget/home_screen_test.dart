import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:go_router/go_router.dart';
import 'package:shared_preferences/shared_preferences.dart';

import 'package:blocus_frontend/domain/providers.dart';
import 'package:blocus_frontend/presentation/screens/home_screen.dart';

void main() {
  late SharedPreferences prefs;

  setUp(() async {
    SharedPreferences.setMockInitialValues({});
    prefs = await SharedPreferences.getInstance();
  });

  Widget buildApp() {
    final router = GoRouter(
      routes: [
        GoRoute(
          path: '/',
          builder: (_, _) => const HomeScreen(),
        ),
        GoRoute(
          path: '/game/:gameId',
          builder: (_, _) => const Scaffold(body: Text('Game')),
        ),
      ],
    );

    return ProviderScope(
      overrides: [
        // Synchronous override — prefs is already loaded in setUp.
        sharedPrefsProvider.overrideWithValue(prefs),
      ],
      child: MaterialApp.router(routerConfig: router),
    );
  }

  testWidgets('HomeScreen renders header and tabs', (tester) async {
    await tester.pumpWidget(buildApp());
    await tester.pumpAndSettle();

    expect(find.text('Blocus'), findsOneWidget);
    expect(find.text('Create Game'), findsWidgets);
    expect(find.text('Join Game'), findsWidgets);
  });

  testWidgets('HomeScreen shows server URL field', (tester) async {
    await tester.pumpWidget(buildApp());
    await tester.pumpAndSettle();

    expect(find.byType(TextFormField), findsWidgets);
    expect(find.text('Server URL'), findsOneWidget);
  });

  testWidgets('HomeScreen shows mode dropdown', (tester) async {
    await tester.pumpWidget(buildApp());
    await tester.pumpAndSettle();

    expect(find.text('Mode'), findsOneWidget);
  });
}
