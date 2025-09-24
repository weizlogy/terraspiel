
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

export const handleSoil = ({
  grid,
  lastMoveGrid,
  colorGrid,
  newGrid,
  newLastMoveGrid,
  newColorGrid,
  moved,
  x,
  y,
  width,
  height,
}: BehaviorContext): void => {
  const color = colorGrid[y][x];
  let hasMoved = false;

  // 4. Check for water interaction and convert to mud (both soil and water)
  let waterNearby = false;
  let waterY = -1; // Position of the nearby water
  let waterX = -1;

  if (y + 1 < height && grid[y + 1][x] === 'WATER' && !moved[y + 1][x]) {
    waterNearby = true;
    waterY = y + 1;
    waterX = x;
  } else if (y > 0 && grid[y - 1][x] === 'WATER' && !moved[y - 1][x]) {
    waterNearby = true;
    waterY = y - 1;
    waterX = x;
  } else if (x + 1 < width && grid[y][x + 1] === 'WATER' && !moved[y][x + 1]) {
    waterNearby = true;
    waterY = y;
    waterX = x + 1;
  } else if (x > 0 && grid[y][x - 1] === 'WATER' && !moved[y][x - 1]) {
    waterNearby = true;
    waterY = y;
    waterX = x - 1;
  }

  if (waterNearby) {
    newGrid[y][x] = 'MUD';
    newColorGrid[y][x] = ELEMENTS.MUD.color;
    moved[y][x] = true; // Mark as moved to prevent further movement
    hasMoved = true;
    // Consume the nearby water (convert to empty)
    newGrid[waterY][waterX] = 'EMPTY';
    newColorGrid[waterY][waterX] = ELEMENTS.EMPTY.color;
    moved[waterY][waterX] = true; // Mark water as moved to prevent further movement
  }
  // 1. Try moving down into empty space (only if not converted to mud)
  else if (y + 1 < height && grid[y + 1][x] === 'EMPTY' && !moved[y + 1][x]) {
    newGrid[y][x] = 'EMPTY';
    newColorGrid[y][x] = ELEMENTS.EMPTY.color;
    newGrid[y + 1][x] = 'SOIL';
    newColorGrid[y + 1][x] = color;
    moved[y][x] = true;
    moved[y + 1][x] = true;
    newLastMoveGrid[y + 1][x] = 'DOWN';
    hasMoved = true;
  }
  // 2. Try swapping with water below (only if not converted to mud)
  else if (y + 1 < height && grid[y + 1][x] === 'WATER' && !moved[y + 1][x]) {
    const waterColor = colorGrid[y + 1][x];
    // Swap elements
    newGrid[y][x] = 'WATER';
    newGrid[y + 1][x] = 'SOIL';
    // Swap colors
    newColorGrid[y][x] = waterColor;
    newColorGrid[y + 1][x] = color;

    moved[y][x] = true;
    moved[y + 1][x] = true;
    newLastMoveGrid[y + 1][x] = 'DOWN'; // SOIL moved down
    hasMoved = true;
  }
  // 3. Try moving diagonally down (only if not converted to mud)
  else if (y + 1 < height) {
    const lastMove = lastMoveGrid[y][x];
    const inertiaChance = 0.75; // 75% chance to follow inertia

    let directions = [-1, 1]; // Default: left, right
    if (Math.random() > 0.5) directions.reverse();

    // Apply inertia: If last move was diagonal, try that direction first
    if (lastMove === 'DOWN_LEFT' && Math.random() < inertiaChance) {
      directions = [-1, 1];
    } else if (lastMove === 'DOWN_RIGHT' && Math.random() < inertiaChance) {
      directions = [1, -1];
    }

    for (const dx of directions) {
      if (x + dx >= 0 && x + dx < width &&
          grid[y + 1][x + dx] === 'EMPTY' && !moved[y + 1][x + dx]) {
        newGrid[y][x] = 'EMPTY';
        newColorGrid[y][x] = ELEMENTS.EMPTY.color;
        newGrid[y + 1][x + dx] = 'SOIL';
        newColorGrid[y + 1][x + dx] = color;
        moved[y][x] = true;
        moved[y + 1][x + dx] = true;
        newLastMoveGrid[y + 1][x + dx] = dx === -1 ? 'DOWN_LEFT' : 'DOWN_RIGHT';
        hasMoved = true;
        break;
      }
    }
  }

  // 4. Try to slip sideways if on a peak (only if not converted to mud)
  if (!hasMoved && Math.random() < 0.3) { // 30% chance to slip
    // Check if the particle is on a peak (empty on both sides)
    if (x > 0 && x < width - 1 && grid[y][x - 1] === 'EMPTY' && grid[y][x + 1] === 'EMPTY') {
      const slipDirection = Math.random() > 0.5 ? 1 : -1;

      if (!moved[y][x + slipDirection]) {
        newGrid[y][x] = 'EMPTY';
        newColorGrid[y][x] = ELEMENTS.EMPTY.color;
        newGrid[y][x + slipDirection] = 'SOIL';
        newColorGrid[y][x + slipDirection] = color;
        moved[y][x] = true;
        moved[y][x + slipDirection] = true;
        newLastMoveGrid[y][x + slipDirection] = slipDirection === -1 ? 'LEFT' : 'RIGHT';
        hasMoved = true;
      }
    }
  }
};
