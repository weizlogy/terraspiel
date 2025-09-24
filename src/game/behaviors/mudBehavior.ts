
import { ELEMENTS, type ElementName, type MoveDirection } from "../../types/elements";

interface BehaviorContext {
  grid: ElementName[][];
  lastMoveGrid: MoveDirection[][];
  colorGrid: string[][];
  newGrid: ElementName[][];
  newLastMoveGrid: MoveDirection[][];
  newColorGrid: string[][];
  moved: boolean[][];
  x: number;
  y: number;
  width: number;
  height: number;
}

export const handleMud = ({
  grid,
  colorGrid,
  newGrid,
  newColorGrid,
  moved,
  x,
  y,
  width,
  height,
}: BehaviorContext): void => {
  const color = colorGrid[y][x];
  let hasMoved = false;

  // 1. Try moving down
  if (!hasMoved && y + 1 < height && (grid[y + 1][x] === 'EMPTY' || grid[y + 1][x] === 'WATER') && !moved[y + 1][x]) {
    // Swap with water if below
    if (grid[y + 1][x] === 'WATER') {
      const waterColor = colorGrid[y + 1][x];
      newGrid[y][x] = 'WATER';
      newGrid[y + 1][x] = 'MUD';
      newColorGrid[y][x] = waterColor;
      newColorGrid[y + 1][x] = color;
    } else {
      // Move down to empty space
      newGrid[y][x] = 'EMPTY';
      newColorGrid[y][x] = ELEMENTS.EMPTY.color;
      newGrid[y + 1][x] = 'MUD';
      newColorGrid[y + 1][x] = color;
    }
    moved[y][x] = true;
    moved[y + 1][x] = true;
    hasMoved = true;
  }

  // 2. Try moving diagonally down (randomize direction to avoid bias)
  if (!hasMoved && y + 1 < height) {
    const diagonalDirections = [-1, 1]; // Left-down and right-down
    if (Math.random() > 0.5) diagonalDirections.reverse();

    for (const dx of diagonalDirections) {
      if (x + dx >= 0 && x + dx < width &&
          (grid[y + 1][x + dx] === 'EMPTY' || grid[y + 1][x + dx] === 'WATER') && !moved[y + 1][x + dx]) {
        // Swap with water if diagonally below
        if (grid[y + 1][x + dx] === 'WATER') {
          const waterColor = colorGrid[y + 1][x + dx];
          newGrid[y][x] = 'WATER';
          newGrid[y + 1][x + dx] = 'MUD';
          newColorGrid[y][x] = waterColor;
          newColorGrid[y + 1][x + dx] = color;
        } else {
          // Move diagonally down to empty space
          newGrid[y][x] = 'EMPTY';
          newColorGrid[y][x] = ELEMENTS.EMPTY.color;
          newGrid[y + 1][x + dx] = 'MUD';
          newColorGrid[y + 1][x + dx] = color;
        }
        moved[y][x] = true;
        moved[y + 1][x + dx] = true;
        hasMoved = true;
        break;
      }
    }
  }

  // 3. Try moving sideways with less horizontal stability compared to water
  if (!hasMoved) {
    // Mud spreads sideways but doesn't maintain a perfect horizontal level
    const leftX = x - 1;
    const rightX = x + 1;

    // Check availability of left and right positions
    const canGoLeft = leftX >= 0 && grid[y][leftX] === 'EMPTY' && !moved[y][leftX];
    const canGoRight = rightX < width && grid[y][rightX] === 'EMPTY' && !moved[y][rightX];

    if (canGoLeft && canGoRight) {
      // Mud moves sideways more randomly without trying to balance levels like water
      const direction = Math.random() > 0.5 ? leftX : rightX;
      newGrid[y][x] = 'EMPTY';
      newColorGrid[y][x] = ELEMENTS.EMPTY.color;
      newGrid[y][direction] = 'MUD';
      newColorGrid[y][direction] = color;
      moved[y][x] = true;
      moved[y][direction] = true;
      hasMoved = true;
    } else if (canGoLeft) {
      newGrid[y][x] = 'EMPTY';
      newColorGrid[y][x] = ELEMENTS.EMPTY.color;
      newGrid[y][leftX] = 'MUD';
      newColorGrid[y][leftX] = color;
      moved[y][x] = true;
      moved[y][leftX] = true;
      hasMoved = true;
    } else if (canGoRight) {
      newGrid[y][x] = 'EMPTY';
      newColorGrid[y][x] = ELEMENTS.EMPTY.color;
      newGrid[y][rightX] = 'MUD';
      newColorGrid[y][rightX] = color;
      moved[y][x] = true;
      moved[y][rightX] = true;
      hasMoved = true;
    }
  }
};
