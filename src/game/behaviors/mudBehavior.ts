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

  // 3. Try moving sideways
  if (!hasMoved) {
    const directions = [-1, 1];
    if (Math.random() > 0.5) directions.reverse();

    for (const dx of directions) {
      const nx = x + dx;
      if (nx >= 0 && nx < width && grid[y][nx].type === 'EMPTY' && !moved[y][nx]) {
        newGrid[y][x] = { type: 'EMPTY' };
        newColorGrid[y][x] = elements.EMPTY.color;
        newGrid[y][nx] = { type: 'MUD' };
        newColorGrid[y][nx] = color;
        moved[y][x] = true;
        moved[y][nx] = true;
        hasMoved = true;
        break;
      }
    }
  }
};