import { ELEMENTS, type Cell } from "../../types/elements";

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

export const handleCloud = ({
  grid,
  colorGrid,
  newGrid,
  newColorGrid,
  moved,
  x,
  y,
  width,
  // height,
}: BehaviorContext): void => {
  const color = colorGrid[y][x];
  let hasMoved = false;

  // Movement priority: up > diagonally up > sideways, with some randomness
  const upwardChance = 0.7; // High chance to move up
  const spreadChance = 0.5; // Moderate chance to spread

  // 1. Try moving up
  if (y > 0 && !hasMoved && Math.random() < upwardChance) {
    if (grid[y - 1][x].type === 'EMPTY' && !moved[y - 1][x]) {
      newGrid[y][x] = { type: 'EMPTY' };
      newColorGrid[y][x] = ELEMENTS.EMPTY.color;
      newGrid[y - 1][x] = { type: 'CLOUD' };
      newColorGrid[y - 1][x] = color;
      moved[y][x] = true;
      moved[y - 1][x] = true;
      hasMoved = true;
    }
  }

  // 2. Try moving diagonally up
  if (y > 0 && !hasMoved && Math.random() < spreadChance) {
    const diagonalDirections = [-1, 1];
    if (Math.random() > 0.5) diagonalDirections.reverse();

    for (const dx of diagonalDirections) {
      const nx = x + dx;
      if (nx >= 0 && nx < width && grid[y - 1][nx].type === 'EMPTY' && !moved[y - 1][nx]) {
        newGrid[y][x] = { type: 'EMPTY' };
        newColorGrid[y][x] = ELEMENTS.EMPTY.color;
        newGrid[y - 1][nx] = { type: 'CLOUD' };
        newColorGrid[y - 1][nx] = color;
        moved[y][x] = true;
        moved[y - 1][nx] = true;
        hasMoved = true;
        break;
      }
    }
  }

  // 3. Try moving sideways
  if (!hasMoved && Math.random() < spreadChance) {
    const directions = [-1, 1];
    if (Math.random() > 0.5) directions.reverse();

    for (const dx of directions) {
      const nx = x + dx;
      if (nx >= 0 && nx < width && grid[y][nx].type === 'EMPTY' && !moved[y][nx]) {
        newGrid[y][x] = { type: 'EMPTY' };
        newColorGrid[y][x] = ELEMENTS.EMPTY.color;
        newGrid[y][nx] = { type: 'CLOUD' };
        newColorGrid[y][nx] = color;
        moved[y][x] = true;
        moved[y][nx] = true;
        hasMoved = true;
        break;
      }
    }
  }
};