import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../../../data/websocket_service.dart';
import '../../../domain/providers.dart';

/// Thin animated banner that appears at the top of the screen when the
/// WebSocket connection is reconnecting.
///
/// - Hidden when connected or disconnected intentionally.
/// - Shown with a yellow warning colour during [WsConnectionState.reconnecting].
class ConnectionStatusBanner extends ConsumerWidget {
  const ConnectionStatusBanner({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final stateAsync = ref.watch(connectionStateProvider);

    final isReconnecting =
        stateAsync.valueOrNull == WsConnectionState.reconnecting;

    return AnimatedContainer(
      duration: const Duration(milliseconds: 300),
      height: isReconnecting ? 32 : 0,
      color: Colors.amber.shade700,
      alignment: Alignment.center,
      child: isReconnecting
          ? Row(
              mainAxisAlignment: MainAxisAlignment.center,
              children: [
                const SizedBox(
                  width: 14,
                  height: 14,
                  child: CircularProgressIndicator(
                    strokeWidth: 2,
                    valueColor:
                        AlwaysStoppedAnimation<Color>(Colors.white),
                  ),
                ),
                const SizedBox(width: 8),
                Text(
                  'Reconnecting…',
                  style: Theme.of(context).textTheme.labelSmall?.copyWith(
                        color: Colors.white,
                        fontWeight: FontWeight.bold,
                      ),
                ),
              ],
            )
          : null,
    );
  }
}
