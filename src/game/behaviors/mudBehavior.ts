import { type Cell } from "../../types/elements";
import useGameStore from "../../stores/gameStore";

interface BehaviorContext {
  grid: Cell[][];
  colorGrid: string[][];
  newGrid: Cell[][];
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
  const elements = useGameStore.getState().elements;
  if (Object.keys(elements).length === 0) return;

  const color = colorGrid[y][x];
  let hasMoved = false;

  // 1. Try moving down (into EMPTY or swapping with WATER)
  if (!hasMoved && y + 1 < height && (grid[y + 1][x].type === 'EMPTY' || grid[y + 1][x].type === 'WATER') && !moved[y + 1][x]) {
    if (grid[y + 1][x].type === 'WATER') {
      const waterColor = colorGrid[y + 1][x];
      newGrid[y][x] = { type: 'WATER' };
      newGrid[y + 1][x] = { type: 'MUD' };
      newColorGrid[y][x] = waterColor;
      newColorGrid[y + 1][x] = color;
    } else {
      newGrid[y][x] = { type: 'EMPTY' };
      newColorGrid[y][x] = elements.EMPTY.color;
      newGrid[y + 1][x] = { type: 'MUD' };
      newColorGrid[y + 1][x] = color;
    }
    moved[y][x] = true;
    moved[y + 1][x] = true;
    hasMoved = true;
  }

  // 2. Try moving diagonally down
  if (!hasMoved && y + 1 < height) {
    const diagonalDirections = [-1, 1];
    if (Math.random() > 0.5) diagonalDirections.reverse();

    for (const dx of diagonalDirections) {
      if (x + dx >= 0 && x + dx < width &&
          (grid[y + 1][x + dx].type === 'EMPTY' || grid[y + 1][x + dx].type === 'WATER') && !moved[y + 1][x + dx]) {
        if (grid[y + 1][x + dx].type === 'WATER') {
          const waterColor = colorGrid[y + 1][x + dx];
          newGrid[y][x] = { type: 'WATER' };
          newGrid[y + 1][x + dx] = { type: 'MUD' };
          newColorGrid[y][x] = waterColor;
          newColorGrid[y + 1][x + dx] = color;
        } else {
          newGrid[y][x] = { type: 'EMPTY' };
          newColorGrid[y][x] = elements.EMPTY.color;
          newGrid[y + 1][x + dx] = { type: 'MUD' };
          newColorGrid[y + 1][x + dx] = color;
        }
        moved[y][x] = true;
        moved[y + 1][x + dx] = true;
        hasMoved = true;
        break;
      }
    }
  }

  // 3. Try moving sideways with horizontal stability in mind (adapted from handleWater)
  if (!hasMoved) {
    const leftX = x - 1;
    const rightX = x + 1;

    // Check availability of left and right positions
    const canGoLeft = leftX >= 0 && grid[y][leftX].type === 'EMPTY' && !moved[y][leftX];
    const canGoRight = rightX < width && grid[y][rightX].type === 'EMPTY' && !moved[y][rightX];

    if (canGoLeft && canGoRight) {
      // When both sides are available, check which side has more empty space below
      // to spread mud more evenly and maintain horizontal level
      let leftOpenSpaces = 0;
      let rightOpenSpaces = 0;

      // Count empty spaces below the potential left position
      for (let testY = y; testY < height; testY++) {
        if (grid[testY][leftX].type === 'EMPTY') {
          leftOpenSpaces++;
        } else {
          break;
        }
      }

      // Count empty spaces below the potential right position
      for (let testY = y; testY < height; testY++) {
        if (grid[testY][rightX].type === 'EMPTY') {
          rightOpenSpaces++;
        } else {
          break;
        }
      }

      // Move to the side with more open space below, or random if equal
      if (leftOpenSpaces > rightOpenSpaces) {
        newGrid[y][x] = { type: 'EMPTY' };
        newColorGrid[y][x] = elements.EMPTY.color;
        newGrid[y][leftX] = { type: 'MUD' };
        newColorGrid[y][leftX] = color;
        moved[y][x] = true;
        moved[y][leftX] = true;
        hasMoved = true;
      } else if (rightOpenSpaces > leftOpenSpaces) {
        newGrid[y][x] = { type: 'EMPTY' };
        newColorGrid[y][x] = elements.EMPTY.color;
        newGrid[y][rightX] = { type: 'MUD' };
        newColorGrid[y][rightX] = color;
        moved[y][x] = true;
        moved[y][rightX] = true;
        hasMoved = true;
      } else {
        // If equal, move in random direction
        const direction = Math.random() > 0.5 ? leftX : rightX;
        newGrid[y][x] = { type: 'EMPTY' };
        newColorGrid[y][x] = elements.EMPTY.color;
        newGrid[y][direction] = { type: 'MUD' };
        newColorGrid[y][direction] = color;
        moved[y][x] = true;
        moved[y][direction] = true;
        hasMoved = true;
      }
    } else if (canGoLeft) {
      newGrid[y][x] = { type: 'EMPTY' };
        newColorGrid[y][x] = elements.EMPTY.color;
        newGrid[y][leftX] = { type: 'MUD' };
        newColorGrid[y][leftX] = color;
        moved[y][x] = true;
        moved[y][leftX] = true;
        hasMoved = true;
    } else if (canGoRight) {
      newGrid[y][x] = { type: 'EMPTY' };
        newColorGrid[y][x] = elements.EMPTY.color;
        newGrid[y][rightX] = { type: 'MUD' };
        newColorGrid[y][rightX] = color;
        moved[y][x] = true;
        moved[y][rightX] = true;
        hasMoved = true;
    }
  }
};