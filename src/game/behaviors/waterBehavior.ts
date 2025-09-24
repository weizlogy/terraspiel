
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

export const handleWater = ({
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
  // Movement priority: down > diagonally down > sideways
  let hasMoved = false;

  // 1. Try moving down
  if (!hasMoved && y + 1 < height && grid[y + 1][x] === 'EMPTY' && !moved[y + 1][x]) {
    newGrid[y][x] = 'EMPTY';
    newColorGrid[y][x] = ELEMENTS.EMPTY.color;
    newGrid[y + 1][x] = 'WATER';
    newColorGrid[y + 1][x] = color;
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
          grid[y + 1][x + dx] === 'EMPTY' && !moved[y + 1][x + dx]) {
        newGrid[y][x] = 'EMPTY';
        newColorGrid[y][x] = ELEMENTS.EMPTY.color;
        newGrid[y + 1][x + dx] = 'WATER';
        newColorGrid[y + 1][x + dx] = color;
        moved[y][x] = true;
        moved[y + 1][x + dx] = true;
        hasMoved = true;
        break;
      }
    }
  }

  // 3. Try moving sideways with horizontal stability in mind
  if (!hasMoved) {
    // Instead of random direction, check which side has more space for water to spread evenly
    const leftX = x - 1;
    const rightX = x + 1;

    // Check availability of left and right positions
    const canGoLeft = leftX >= 0 && grid[y][leftX] === 'EMPTY' && !moved[y][leftX];
    const canGoRight = rightX < width && grid[y][rightX] === 'EMPTY' && !moved[y][rightX];

    if (canGoLeft && canGoRight) {
      // When both sides are available, check which side has more empty space below
      // to spread water more evenly and maintain horizontal level
      let leftOpenSpaces = 0;
      let rightOpenSpaces = 0;

      // Count empty spaces below the potential left position
      for (let testY = y; testY < height; testY++) {
        if (grid[testY][leftX] === 'EMPTY') {
          leftOpenSpaces++;
        } else {
          break;
        }
      }

      // Count empty spaces below the potential right position
      for (let testY = y; testY < height; testY++) {
        if (grid[testY][rightX] === 'EMPTY') {
          rightOpenSpaces++;
        } else {
          break;
        }
      }

      // Move to the side with more open space below, or random if equal
      if (leftOpenSpaces > rightOpenSpaces) {
        newGrid[y][x] = 'EMPTY';
        newColorGrid[y][x] = ELEMENTS.EMPTY.color;
        newGrid[y][leftX] = 'WATER';
        newColorGrid[y][leftX] = color;
        moved[y][x] = true;
        moved[y][leftX] = true;
        hasMoved = true;
      } else if (rightOpenSpaces > leftOpenSpaces) {
        newGrid[y][x] = 'EMPTY';
        newColorGrid[y][x] = ELEMENTS.EMPTY.color;
        newGrid[y][rightX] = 'WATER';
        newColorGrid[y][rightX] = color;
        moved[y][x] = true;
        moved[y][rightX] = true;
        hasMoved = true;
      } else {
        // If equal, move in random direction
        const direction = Math.random() > 0.5 ? leftX : rightX;
        newGrid[y][x] = 'EMPTY';
        newColorGrid[y][x] = ELEMENTS.EMPTY.color;
        newGrid[y][direction] = 'WATER';
        newColorGrid[y][direction] = color;
        moved[y][x] = true;
        moved[y][direction] = true;
        hasMoved = true;
      }
    } else if (canGoLeft) {
      newGrid[y][x] = 'EMPTY';
      newColorGrid[y][x] = ELEMENTS.EMPTY.color;
      newGrid[y][leftX] = 'WATER';
      newColorGrid[y][leftX] = color;
      moved[y][x] = true;
      moved[y][leftX] = true;
      hasMoved = true;
    } else if (canGoRight) {
      newGrid[y][x] = 'EMPTY';
      newColorGrid[y][x] = ELEMENTS.EMPTY.color;
      newGrid[y][rightX] = 'WATER';
      newColorGrid[y][rightX] = color;
      moved[y][x] = true;
      moved[y][rightX] = true;
      hasMoved = true;
    }
  }
};
