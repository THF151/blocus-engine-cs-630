import 'package:flutter_riverpod/flutter_riverpod.dart';
import '../core/constants.dart';

// ──────────────────────────────────────────────────────────────────────────────
// State
// ──────────────────────────────────────────────────────────────────────────────

/// Tracks which piece the local player has selected and in which orientation.
class PieceSelectionState {
  /// The engine piece_id of the selected piece, or null if none is selected.
  final int? selectedPieceId;

  /// Index into [PieceDefinition.orientations] for the selected piece.
  final int orientationIndex;

  const PieceSelectionState({this.selectedPieceId, this.orientationIndex = 0});

  bool get hasPieceSelected => selectedPieceId != null;

  /// Returns the current piece's cells in the active orientation, or null.
  OrientationCells? get currentCells {
    if (selectedPieceId == null) return null;
    return pieceById(selectedPieceId!).cellsForOrientation(orientationIndex);
  }

  /// Returns how many orientations the selected piece has.
  int get orientationCount {
    if (selectedPieceId == null) return 1;
    return pieceById(selectedPieceId!).orientations.length;
  }

  PieceSelectionState copyWith({
    int? selectedPieceId,
    int? orientationIndex,
    bool clearSelection = false,
  }) => PieceSelectionState(
    selectedPieceId:
        clearSelection ? null : selectedPieceId ?? this.selectedPieceId,
    orientationIndex: orientationIndex ?? this.orientationIndex,
  );
}

// ──────────────────────────────────────────────────────────────────────────────
// Notifier
// ──────────────────────────────────────────────────────────────────────────────

/// Manages piece selection, rotation, and flip for the local player.
class PieceSelectionNotifier extends StateNotifier<PieceSelectionState> {
  PieceSelectionNotifier() : super(const PieceSelectionState());

  // ── Selection ───────────────────────────────────────────────────────────────

  /// Selects [pieceId] and resets to orientation 0.
  ///
  /// Tapping the same piece a second time deselects it.
  void selectPiece(int pieceId) {
    if (state.selectedPieceId == pieceId) {
      clearSelection();
    } else {
      state = PieceSelectionState(
        selectedPieceId: pieceId,
        orientationIndex: 0,
      );
    }
  }

  /// Deselects the current piece.
  void clearSelection() {
    state = const PieceSelectionState();
  }

  // ── Orientation ─────────────────────────────────────────────────────────────

  /// Advances to the next orientation (wraps around).
  void rotateNext() {
    if (state.selectedPieceId == null) return;
    final count = state.orientationCount;
    state = state.copyWith(
      orientationIndex: (state.orientationIndex + 1) % count,
    );
  }

  /// Goes to the previous orientation (wraps around).
  void rotatePrev() {
    if (state.selectedPieceId == null) return;
    final count = state.orientationCount;
    state = state.copyWith(
      orientationIndex: (state.orientationIndex - 1 + count) % count,
    );
  }

  /// Jumps to an explicit [orientationIndex].
  void setOrientation(int orientationIndex) {
    if (state.selectedPieceId == null) return;
    final count = state.orientationCount;
    state = state.copyWith(
      orientationIndex: orientationIndex.clamp(0, count - 1),
    );
  }

  /// Called after a move is successfully placed so the selection is reset.
  void onMovePlaced() => clearSelection();
}
