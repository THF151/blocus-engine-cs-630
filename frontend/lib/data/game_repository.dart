import 'package:uuid/uuid.dart';
import 'websocket_service.dart';

/// Translates high-level game actions into backend WebSocket frames.
///
/// All methods delegate to [WebSocketService.send] with the correct
/// `action` string and payload that satisfies the backend's Pydantic schemas
/// (see `backend/src/blocus_backend/schemas.py`).
class GameRepository {
  final WebSocketService _ws;
  final Uuid _uuid;

  GameRepository(this._ws) : _uuid = const Uuid();

  // ── Game creation ───────────────────────────────────────────────────────────

  /// Creates a **two-player** game where each human controls two colours.
  void createTwoPlayerGame({
    required String gameId,
    required String blueRedPlayerId,
    required String yellowGreenPlayerId,
    String scoring = 'basic',
  }) {
    _ws.send('create_game', {
      'game_id': gameId,
      'mode': 'two_player',
      'scoring': scoring,
      'players': {
        'blue_red': blueRedPlayerId,
        'yellow_green': yellowGreenPlayerId,
      },
    });
  }

  /// Creates a **three-player** game.
  ///
  /// [sharedGreenPlayers] is the ordered list of player IDs that share the
  /// green colour in rotation.
  void createThreePlayerGame({
    required String gameId,
    required String bluePlayerId,
    required String yellowPlayerId,
    required String redPlayerId,
    required List<String> sharedGreenPlayers,
    String scoring = 'basic',
  }) {
    _ws.send('create_game', {
      'game_id': gameId,
      'mode': 'three_player',
      'scoring': scoring,
      'players': {
        'blue': bluePlayerId,
        'yellow': yellowPlayerId,
        'red': redPlayerId,
        'shared_green': sharedGreenPlayers,
      },
    });
  }

  /// Creates a **four-player** Classic game.
  void createFourPlayerGame({
    required String gameId,
    required String bluePlayerId,
    required String yellowPlayerId,
    required String redPlayerId,
    required String greenPlayerId,
    String firstColor = 'blue',
    String scoring = 'basic',
  }) {
    _ws.send('create_game', {
      'game_id': gameId,
      'mode': 'four_player',
      'scoring': scoring,
      'first_color': firstColor,
      'players': {
        'blue': bluePlayerId,
        'yellow': yellowPlayerId,
        'red': redPlayerId,
        'green': greenPlayerId,
      },
    });
  }

  /// Creates a **Duo** game (14×14, advanced scoring, two players).
  void createDuoGame({
    required String gameId,
    required String blackPlayerId,
    required String whitePlayerId,
    String firstColor = 'black',
  }) {
    _ws.send('create_game', {
      'game_id': gameId,
      'mode': 'duo',
      'scoring': 'advanced', // Duo is always advanced
      'first_color': firstColor,
      'players': {
        'black': blackPlayerId,
        'white': whitePlayerId,
      },
    });
  }

  // ── Subscription ────────────────────────────────────────────────────────────

  /// Subscribes the connection to a game.
  ///
  /// Pass [playerId] to claim the seat for that player (needed before sending
  /// moves).  Omit to subscribe as spectator.
  void subscribeGame(String gameId, {String? playerId}) {
    _ws.send('subscribe_game', {
      'game_id': gameId,
      if (playerId != null) 'player_id': playerId,
    });
  }

  /// Alias for [subscribeGame] with a player seat.
  void joinGame(String gameId, String playerId) =>
      subscribeGame(gameId, playerId: playerId);

  // ── State queries ───────────────────────────────────────────────────────────

  /// Re-fetches the current state snapshot (unicast).
  void requestState(String gameId) {
    _ws.send('request_state', {'game_id': gameId});
  }

  /// Fetches all legal moves for [playerId] controlling [color].
  ///
  /// The response arrives as a [LegalMovesMessage] on the WS stream.
  void requestLegalMoves(String gameId, String playerId, String color) {
    _ws.send('request_legal_moves', {
      'game_id': gameId,
      'player_id': playerId,
      'color': color,
    });
  }

  /// Requests the final score.  Only meaningful once the game is finished.
  void requestScore(String gameId) {
    _ws.send('request_score', {'game_id': gameId});
  }

  // ── Actions ─────────────────────────────────────────────────────────────────

  /// Places [pieceId] in [orientationId] at board position ([row], [col]).
  ///
  /// [row] / [col] refer to the top-left corner of the piece's bounding box.
  /// A unique [command_id] (UUID) is generated automatically for idempotency.
  void placeMove({
    required String gameId,
    required String playerId,
    required String color,
    required int pieceId,
    required int orientationId,
    required int row,
    required int col,
  }) {
    _ws.send('place_move', {
      'game_id': gameId,
      'command_id': _uuid.v4(),
      'player_id': playerId,
      'color': color,
      'piece_id': pieceId,
      'orientation_id': orientationId,
      'row': row,
      'col': col,
    });
  }

  /// Passes the current player's turn.
  void passMove({
    required String gameId,
    required String playerId,
    required String color,
  }) {
    _ws.send('pass_move', {
      'game_id': gameId,
      'command_id': _uuid.v4(),
      'player_id': playerId,
      'color': color,
    });
  }

  // ── AI ──────────────────────────────────────────────────────────────────────

  /// Attaches an AI player for the given [color] seat.
  ///
  /// The backend automatically makes moves on the AI's behalf.
  void attachAi({
    required String gameId,
    required String playerId,
    required String color,
  }) {
    _ws.send('attach_ai', {
      'game_id': gameId,
      'player_id': playerId,
      'color': color,
    });
  }

  // ── Helpers ─────────────────────────────────────────────────────────────────

  /// Generates a new UUID that can be used as a `game_id` when creating games.
  String newGameId() => _uuid.v4();
}
