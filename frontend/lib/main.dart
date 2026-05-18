import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:shared_preferences/shared_preferences.dart';
import 'app.dart';
import 'domain/providers.dart';

void main() async {
  WidgetsFlutterBinding.ensureInitialized();
  final prefs = await SharedPreferences.getInstance();
  runApp(
    ProviderScope(
      overrides: [
        // Synchronous override so preferencesServiceProvider is available
        // immediately on first access — no async gap.
        sharedPrefsProvider.overrideWithValue(prefs),
      ],
      child: const BlocusApp(),
    ),
  );
}
