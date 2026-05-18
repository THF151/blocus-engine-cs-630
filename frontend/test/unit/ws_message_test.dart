import 'package:flutter_test/flutter_test.dart';

import 'package:blocus_frontend/data/models/ws_message.dart';

void main() {
  group('WsMessage.fromJson', () {
    test('parses game_created', () {
      final msg = WsMessage.fromJson({
        'type': 'game_created',
        'game_id': 'abc-123',
      });
      expect(msg, isA<GameCreatedMessage>());
      expect((msg as GameCreatedMessage).gameId, 'abc-123');
    });

    test('parses state_snapshot', () {
      final msg = WsMessage.fromJson({
        'type': 'state_snapshot',
        'game_id': 'g1',
        'state': {
          'mode': 'duo',
          'scoring': 'basic',
          'current_color': 'black',
          'turn_order': ['black', 'white'],
          'board_cells': [],
          'board_counts': [],
          'pieces_remaining': {},
          'is_finished': false,
        },
      });
      expect(msg, isA<StateSnapshotMessage>());
      final snap = msg as StateSnapshotMessage;
      expect(snap.stateJson['current_color'], 'black');
      expect(snap.stateJson['is_finished'], false);
    });

    test('parses game_finished', () {
      final msg = WsMessage.fromJson({
        'type': 'game_finished',
        'game_id': 'g1',
      });
      expect(msg, isA<GameFinishedMessage>());
    });

    test('returns UnknownMessage for unknown type', () {
      final msg = WsMessage.fromJson({'type': 'totally_unknown'});
      expect(msg, isA<UnknownMessage>());
    });

    test('parses error', () {
      final msg = WsMessage.fromJson({
        'type': 'error',
        'message': 'something went wrong',
      });
      expect(msg, isA<WsErrorMessage>());
      expect((msg as WsErrorMessage).message, 'something went wrong');
    });

    test('parses legal_moves', () {
      final msg = WsMessage.fromJson({
        'type': 'legal_moves',
        'game_id': 'g1',
        'player_id': 'alice',
        'color': 'blue',
        'moves': [
          {
            'piece_id': 0,
            'orientation_id': 0,
            'row': 0,
            'col': 0,
            'board_index': 0,
            'score_delta': 1,
          }
        ],
      });
      expect(msg, isA<LegalMovesMessage>());
      final lm = msg as LegalMovesMessage;
      expect(lm.moves.length, 1);
      expect(lm.moves.first['piece_id'], 0);
    });
  });
}
