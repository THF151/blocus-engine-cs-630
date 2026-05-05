```rust
pub fn is_legal_placement(
    board: &Board,
    piece: &Piece,
    position: Position,
    player: Player,
) -> bool {
    let mut absolute_cells = Vec::with_capacity(piece.cells.len());

    for relative in &piece.cells {
        let absolute = Position {
            row: position.row + relative.row,
            col: position.col + relative.col,
        };

        // 1. Piece must fit entirely inside the board.
        if !Board::in_bounds(absolute) {
            return false;
        }

        // 2. Piece must not overlap any existing piece.
        if board.get(absolute).is_some() {
            return false;
        }

        absolute_cells.push(absolute);
    }

    let is_first_move = !board.has_player_piece(player);

    if is_first_move {
        let start_square = player.start_square();

        // First move must cover the player's starting square.
        return absolute_cells.contains(&start_square);
    }

    let edge_offsets = [
        (-1, 0),
        (1, 0),
        (0, -1),
        (0, 1),
    ];

    let corner_offsets = [
        (-1, -1),
        (-1, 1),
        (1, -1),
        (1, 1),
    ];

    let mut has_same_color_corner_contact = false;

    for cell in &absolute_cells {
        // 3. Cannot edge-touch a same-color piece.
        for (dr, dc) in edge_offsets {
            let neighbor = Position {
                row: cell.row + dr,
                col: cell.col + dc,
            };

            if Board::in_bounds(neighbor) && board.get(neighbor) == Some(player) {
                return false;
            }
        }

        // 4. Must corner-touch at least one same-color piece.
        for (dr, dc) in corner_offsets {
            let neighbor = Position {
                row: cell.row + dr,
                col: cell.col + dc,
            };

            if Board::in_bounds(neighbor) && board.get(neighbor) == Some(player) {
                has_same_color_corner_contact = true;
            }
        }
    }

    has_same_color_corner_contact
}
```