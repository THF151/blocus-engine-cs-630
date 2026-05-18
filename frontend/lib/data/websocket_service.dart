import 'dart:async';
import 'dart:convert';
import 'dart:math' as math;

import 'package:flutter/foundation.dart';
import 'package:web_socket_channel/web_socket_channel.dart';

import 'models/ws_message.dart';

// ──────────────────────────────────────────────────────────────────────────────
// Connection state
// ──────────────────────────────────────────────────────────────────────────────

/// Represents the lifecycle of the WebSocket connection.
enum WsConnectionState { disconnected, connecting, connected, reconnecting }

// ──────────────────────────────────────────────────────────────────────────────
// WebSocketService
// ──────────────────────────────────────────────────────────────────────────────

/// Manages a persistent WebSocket connection to the Blocus backend.
///
/// Features:
/// - Automatic reconnect with exponential back-off (max 30 s).
/// - Broadcasts all inbound messages as a [WsMessage] stream.
/// - Exposes [connectionState] so UI can react to connectivity changes.
///
/// Usage:
/// ```dart
/// final svc = WebSocketService();
/// await svc.connect('ws://localhost:8000/ws');
/// svc.messages.listen((msg) { ... });
/// svc.send('subscribe_game', {'game_id': 'x', 'player_id': 'alice'});
/// ```
class WebSocketService {
  static const Duration _maxRetryDelay = Duration(seconds: 30);
  static const int _maxRetryBackoffBase = 2;

  // Output streams
  final _messagesController = StreamController<WsMessage>.broadcast();
  final _connectionStateController =
      StreamController<WsConnectionState>.broadcast();

  // Internal state
  WebSocketChannel? _channel;
  StreamSubscription<dynamic>? _subscription;
  String? _serverUrl;
  Timer? _reconnectTimer;
  int _retryCount = 0;
  bool _shouldConnect = false;

  // ── Public API ──────────────────────────────────────────────────────────────

  /// Stream of parsed [WsMessage] objects received from the server.
  Stream<WsMessage> get messages => _messagesController.stream;

  /// Stream reflecting the current connection lifecycle state.
  Stream<WsConnectionState> get connectionState =>
      _connectionStateController.stream;

  /// Initiates a connection to [serverUrl] (must start with `ws://` or
  /// `wss://`).  Reconnects automatically on failure.
  Future<void> connect(String serverUrl) async {
    // Tear down any existing connection first so that orphaned subscriptions
    // cannot fire _onDone → _scheduleReconnect() and overwrite the new channel.
    _reconnectTimer?.cancel();
    _reconnectTimer = null;
    _subscription?.cancel();
    _subscription = null;
    _channel?.sink.close();
    _channel = null;

    _serverUrl = serverUrl;
    _shouldConnect = true;
    _retryCount = 0;
    await _doConnect();
  }

  /// Sends a JSON-encoded action frame to the server.
  ///
  /// Silently no-ops if the channel is not connected.
  void send(String action, Map<String, dynamic> payload) {
    if (_channel == null) return;
    final frame = jsonEncode({'action': action, 'payload': payload});
    try {
      _channel!.sink.add(frame);
    } catch (_) {
      // Connection lost before we could send; the onDone handler will
      // trigger a reconnect.
    }
  }

  /// Gracefully disconnects and stops reconnect attempts.
  void disconnect() {
    _shouldConnect = false;
    _reconnectTimer?.cancel();
    _reconnectTimer = null;
    _subscription?.cancel();
    _subscription = null;
    _channel?.sink.close();
    _channel = null;
    _connectionStateController.add(WsConnectionState.disconnected);
  }

  /// Releases all resources.  Call once when the service is no longer needed.
  void dispose() {
    disconnect();
    _messagesController.close();
    _connectionStateController.close();
  }

  // ── Private helpers ─────────────────────────────────────────────────────────

  Future<void> _doConnect() async {
    if (!_shouldConnect || _serverUrl == null) return;

    _connectionStateController.add(WsConnectionState.connecting);
    try {
      // Validate the URI scheme before attempting to connect, to surface a
      // clear error rather than a cryptic socket exception.
      final uri = Uri.parse(_serverUrl!);
      if (uri.scheme != 'ws' && uri.scheme != 'wss') {
        throw ArgumentError('Server URL must use ws:// or wss:// scheme.');
      }

      _channel = WebSocketChannel.connect(uri);
      await _channel!.ready; // throws on connection failure

      _retryCount = 0;
      _connectionStateController.add(WsConnectionState.connected);

      _subscription = _channel!.stream.listen(
        _onData,
        onError: _onError,
        onDone: _onDone,
        cancelOnError: false,
      );
    } catch (e) {
      debugPrint('[WS] Connection failed: $e');
      _scheduleReconnect();
    }
  }

  void _onData(dynamic data) {
    if (data is! String) return;
    try {
      final json = jsonDecode(data) as Map<String, dynamic>;
      _messagesController.add(WsMessage.fromJson(json));
    } catch (e) {
      // Malformed frame – log and continue.
      debugPrint('[WS] Failed to parse message: $e\nRaw: $data');
    }
  }

  void _onError(Object error, StackTrace stack) {
    debugPrint('[WS] Stream error: $error');
    _scheduleReconnect();
  }

  void _onDone() {
    if (_shouldConnect) {
      debugPrint('[WS] Connection closed unexpectedly, reconnecting…');
      _scheduleReconnect();
    }
  }

  void _scheduleReconnect() {
    if (!_shouldConnect) return;
    _connectionStateController.add(WsConnectionState.reconnecting);

    final delaySeconds = math
        .pow(_maxRetryBackoffBase, _retryCount)
        .toInt()
        .clamp(1, _maxRetryDelay.inSeconds);
    _retryCount++;

    debugPrint('[WS] Reconnecting in ${delaySeconds}s (attempt $_retryCount)');

    _reconnectTimer?.cancel();
    _reconnectTimer = Timer(Duration(seconds: delaySeconds), _doConnect);
  }
}
