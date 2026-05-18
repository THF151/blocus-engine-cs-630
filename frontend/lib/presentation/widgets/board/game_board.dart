import 'dart:math' as math;

import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../../../core/constants.dart';
import '../../../domain/game_notifier.dart';
import '../../../domain/lobby_notifier.dart';
import '../../../domain/providers.dart';
import 'game_board_painter.dart';

/// Renders the Blokus board.
///
/// Interaction model:
/// - **Drag**: A piece widget can be wrapped in a [Draggable<int>] (data =
///   pieceId).  The [DragTarget] here converts the drop position to board
///   (row, col) and calls [GameNotifier.placeMove].
/// - **Tap**: Tapping a board cell tries to place the currently selected piece
///   + orientation from [PieceSelectionNotifier].
/// - **Hover / drag-over**: While the user is dragging over the board the
///   painter shows a semi-transparent preview.
class GameBoard extends ConsumerStatefulWidget {
  final String gameId;
  const GameBoard({super.key, required this.gameId});

  @override
  ConsumerState<GameBoard> createState() => _GameBoardState();
}

class _GameBoardState extends ConsumerState<GameBoard> {
  /// The cell (row, col) the drag cursor is currently over.
  (int, int)? _hoverAnchor;

  /// [RenderBox] used to convert global offsets to local board coordinates.
  final _boardKey = GlobalKey();

  @override
  Widget build(BuildContext context) {
    final gs = ref.watch(gameNotifierProvider(widget.gameId));
    final sel = ref.watch(pieceSelectionProvider);
    final lobby = ref.read(lobbyNotifierProvider);

    // ── Derived values ────────────────────────────────────────────────────

    final boardMatrix = gs.boardMatrix;
    final currentColor = gs.gameState?.currentColor ?? 'blue';
    final boardSize = boardMatrix.length;

    final isLocalTurn =
        gs.gameState != null &&
        lobby.localColors.contains(currentColor) &&
        isHumanControlledTurn(
          mode: lobby.mode,
          currentColor: currentColor,
          sharedColorTurnIndex: gs.gameState?.sharedColorTurnIndex,
        );

    // Start corners for the current board size
    final startCorners =
        boardSize == kDuoBoardSize
            ? kDuoStartCorners.values.toSet()
            : kClassicStartCorners.values.toSet();

    // Compute legal top-left positions for highlighting
    final legalPositions = <(int, int)>{};
    if (sel.hasPieceSelected) {
      for (final m in gs.legalMoves) {
        if (m.pieceId == sel.selectedPieceId &&
            m.orientationId == sel.orientationIndex) {
          legalPositions.add((m.row, m.col));
        }
      }
    }

    // Preview cells while drag is active
    List<(int, int)>? previewCells;
    if (_hoverAnchor != null && sel.hasPieceSelected) {
      final cells = sel.currentCells;
      if (cells != null) {
        previewCells =
            cells
                .map((c) => (_hoverAnchor!.$1 + c.$1, _hoverAnchor!.$2 + c.$2))
                .toList();
      }
    }

    // ── Widget tree ───────────────────────────────────────────────────────

    return RepaintBoundary(
      child: LayoutBuilder(
        builder: (context, constraints) {
          final side = math.min(constraints.maxWidth, constraints.maxHeight);

          // Keep the piece tray cell size in sync with the actual board cells.
          if (boardSize > 0) {
            WidgetsBinding.instance.addPostFrameCallback((_) {
              if (!mounted) return;
              ref.read(boardCellSizeProvider.notifier).state = side / boardSize;
            });
          }

          return Center(
            child: SizedBox(
              width: side,
              height: side,
              child: DragTarget<int>(
                onWillAcceptWithDetails: (_) => isLocalTurn,
                onMove: (details) {
                  final anchor = _globalToCell(details.offset, side, boardSize);
                  if (anchor != _hoverAnchor) {
                    setState(() => _hoverAnchor = anchor);
                  }
                },
                onLeave: (_) => setState(() => _hoverAnchor = null),
                onAcceptWithDetails: (details) {
                  final anchor = _globalToCell(details.offset, side, boardSize);
                  setState(() => _hoverAnchor = null);
                  if (anchor != null) _tryPlaceAt(anchor.$1, anchor.$2, gs);
                },
                builder: (context, candidates, rejected) {
                  return GestureDetector(
                    onTapUp:
                        isLocalTurn
                            ? (d) {
                              final anchor = _localToCell(
                                d.localPosition,
                                side,
                                boardSize,
                              );
                              if (anchor != null) {
                                _tryPlaceAt(anchor.$1, anchor.$2, gs);
                              }
                            }
                            : null,
                    child: CustomPaint(
                      key: _boardKey,
                      size: Size(side, side),
                      painter: GameBoardPainter(
                        boardMatrix: boardMatrix,
                        boardSize: boardSize,
                        startCorners: startCorners,
                        legalPositions: legalPositions,
                        previewCells: previewCells,
                        activeColor: currentColor,
                      ),
                    ),
                  );
                },
              ),
            ),
          );
        },
      ),
    );
  }

  // ── Coordinate helpers ────────────────────────────────────────────────────

  /// Converts a **global** offset (from drag details) to (row, col).
  (int, int)? _globalToCell(Offset global, double side, int boardSize) {
    final box = _boardKey.currentContext?.findRenderObject() as RenderBox?;
    if (box == null) return null;
    final local = box.globalToLocal(global);
    return _localToCell(local, side, boardSize);
  }

  /// Converts a **local** offset to (row, col).
  (int, int)? _localToCell(Offset local, double side, int boardSize) {
    final cellSize = side / boardSize;
    final col = (local.dx / cellSize).floor();
    final row = (local.dy / cellSize).floor();
    if (row < 0 || row >= boardSize || col < 0 || col >= boardSize) {
      return null;
    }
    return (row, col);
  }

  // ── Move execution ────────────────────────────────────────────────────────

  void _tryPlaceAt(int row, int col, GameNotifierState gs) {
    final sel = ref.read(pieceSelectionProvider);
    if (!sel.hasPieceSelected) return;

    // Find the matching legal move
    final match = gs.legalMoves.where(
      (m) =>
          m.pieceId == sel.selectedPieceId &&
          m.orientationId == sel.orientationIndex &&
          m.row == row &&
          m.col == col,
    );

    if (match.isEmpty) return;

    ref
        .read(gameNotifierProvider(widget.gameId).notifier)
        .placeMove(match.first);
    ref.read(pieceSelectionProvider.notifier).onMovePlaced();
  }
}
