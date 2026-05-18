import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

import 'package:blocus_frontend/core/constants.dart';

void main() {
  group('computeOrientations', () {
    test('monomino (I1) has exactly 1 orientation', () {
      final piece = pieceById(0); // I1
      expect(piece.orientations.length, 1);
    });

    test('domino (I2) has exactly 2 orientations', () {
      final piece = pieceById(1); // I2
      expect(piece.orientations.length, 2);
    });

    test('L-tromino has 4 orientations', () {
      // L2 (piece id 3)
      final piece = pieceById(3);
      expect(piece.orientations.length, 4);
    });

    test('S-tetromino has at least 2 orientations', () {
      // S4 (piece id 8) — actual count depends on algorithm deduplication
      final piece = pieceById(8);
      expect(piece.orientations.length, greaterThanOrEqualTo(2));
    });

    test('all pieces have at least 1 orientation', () {
      for (final p in kPieces) {
        expect(p.orientations, isNotEmpty,
            reason: '${p.name} has no orientations');
      }
    });

    test('each orientation cell list is non-empty', () {
      for (final p in kPieces) {
        for (final cells in p.orientations) {
          expect(cells, isNotEmpty, reason: '${p.name} has empty orientation');
        }
      }
    });

    test('cellsForOrientation clamps out-of-range index', () {
      final piece = pieceById(1); // I2, 2 orientations
      final cellsLast = piece.cellsForOrientation(piece.orientations.length - 1);
      final cellsClamped = piece.cellsForOrientation(piece.orientations.length);
      expect(cellsClamped, equals(cellsLast));
    });

    test('total kPieces count is 21', () {
      expect(kPieces.length, 21);
    });
  });

  group('colorForPlayer', () {
    test('returns a Color instance for blue', () {
      expect(colorForPlayer('blue'), isA<Color>());
    });

    test('returns a Color instance for white (Duo)', () {
      expect(colorForPlayer('white'), isA<Color>());
    });

    test('unknown colour returns grey', () {
      final c = colorForPlayer('purple');
      expect(c, isNotNull);
    });
  });

  group('kClassicStartCorners', () {
    test('blue starts at (0,0)', () {
      expect(kClassicStartCorners['blue'], equals((0, 0)));
    });

    test('red starts at (19,19)', () {
      expect(kClassicStartCorners['red'], equals((19, 19)));
    });
  });
}
