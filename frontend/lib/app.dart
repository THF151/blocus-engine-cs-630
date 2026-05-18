import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'core/router.dart';
import 'core/theme.dart';

/// Root application widget.
///
/// Wires the [GoRouter] (via [routerProvider]) and the [AppTheme] together.
/// Both light and dark modes follow Material Design 3 colour schemes derived
/// from the Blokus brand colours.
class BlocusApp extends ConsumerWidget {
  const BlocusApp({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final router = ref.watch(routerProvider);
    return MaterialApp.router(
      title: 'Blocus',
      theme: AppTheme.light(),
      darkTheme: AppTheme.dark(),
      themeMode: ThemeMode.system,
      routerConfig: router,
      debugShowCheckedModeBanner: false,
    );
  }
}
