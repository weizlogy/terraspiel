import { type MoveDirection, type Cell } from "../../types/elements";
import useGameStore from "../../stores/gameStore";

interface BehaviorContext {
  grid: Cell[][];
  lastMoveGrid: MoveDirection[][];
  colorGrid: string[][];
  newGrid: Cell[][];
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
  const elements = useGameStore.getState().elements;
  if (Object.keys(elements).length === 0) return;

  const color = colorGrid[y][x];
  let hasMoved = false;

  // 1. Try moving down into empty space
  if (y + 1 < height && grid[y + 1][x].type === 'EMPTY' && !moved[y + 1][x]) {
    newGrid[y][x] = { type: 'EMPTY' };
    newColorGrid[y][x] = elements.EMPTY.color;
    newGrid[y + 1][x] = { type: 'SOIL' };
    newColorGrid[y + 1][x] = color;
    moved[y][x] = true;
    moved[y + 1][x] = true;
    newLastMoveGrid[y + 1][x] = 'DOWN';
    hasMoved = true;
  }
  // 2. Try swapping with water below
  else if (y + 1 < height && grid[y + 1][x].type === 'WATER' && !moved[y + 1][x]) {
    const waterColor = colorGrid[y + 1][x];
    // Swap elements
    newGrid[y][x] = { type: 'WATER' };
    newGrid[y + 1][x] = { type: 'SOIL' };
    // Swap colors
    newColorGrid[y][x] = waterColor;
    newColorGrid[y + 1][x] = color;

    moved[y][x] = true;
    moved[y + 1][x] = true;
    newLastMoveGrid[y + 1][x] = 'DOWN'; // SOIL moved down
    hasMoved = true;
  }
  // 3. Try moving diagonally down
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
          grid[y + 1][x + dx].type === 'EMPTY' && !moved[y + 1][x + dx]) {
        newGrid[y][x] = { type: 'EMPTY' };
        newColorGrid[y][x] = elements.EMPTY.color;
        newGrid[y + 1][x + dx] = { type: 'SOIL' };
        newColorGrid[y + 1][x + dx] = color;
        moved[y][x] = true;
        moved[y + 1][x + dx] = true;
        newLastMoveGrid[y + 1][x + dx] = dx === -1 ? 'DOWN_LEFT' : 'DOWN_RIGHT';
        hasMoved = true;
        break;
      }
    }
  }

  // 4. Try to slip sideways if on a peak
  if (!hasMoved && Math.random() < 0.3) { // 30% chance to slip
    // Check if the particle is on a peak (empty on both sides)
    if (x > 0 && x < width - 1 && grid[y][x - 1].type === 'EMPTY' && grid[y][x + 1].type === 'EMPTY') {
      const slipDirection = Math.random() > 0.5 ? 1 : -1;

      if (!moved[y][x + slipDirection]) {
        newGrid[y][x] = { type: 'EMPTY' };
        newColorGrid[y][x] = elements.EMPTY.color;
        newGrid[y][x + slipDirection] = { type: 'SOIL' };
        newColorGrid[y][x + slipDirection] = color;
        moved[y][x] = true;
        moved[y][x + slipDirection] = true;
        newLastMoveGrid[y][x + slipDirection] = slipDirection === -1 ? 'LEFT' : 'RIGHT';
        hasMoved = true;
      }
    }
  }
};