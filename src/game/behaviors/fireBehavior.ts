import { type Cell } from "../../types/elements";
import useGameStore from "../../stores/gameStore";

interface BehaviorContext {
  grid: Cell[][];
  newGrid: Cell[][];
  newColorGrid: string[][];
  moved: boolean[][];
  x: number;
  y: number;
  width: number;
  height: number;
}

const BURN_THRESHOLD = 30; // 30フレームで燃える

export const handleFire = ({
  grid,
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

  const directions = [
    [-1, -1], [-1, 0], [-1, 1],
    [0, -1],          [0, 1],
    [1, -1], [1, 0], [1, 1]
  ];

  // Shuffle directions to check a random neighbor first
  for (let i = directions.length - 1; i > 0; i--) {
    const j = Math.floor(Math.random() * (i + 1));
    [directions[i], directions[j]] = [directions[j], directions[i]];
  }

  for (const [dy, dx] of directions) {
    const nx = x + dx;
    const ny = y + dy;

    if (nx >= 0 && nx < width && ny >= 0 && ny < height) {
      if (grid[ny][nx].type === 'SOIL' && !moved[ny][nx]) {
        
        const soilCell = newGrid[ny][nx];
        const currentCounter = soilCell.counter || 0;
        const newCounter = currentCounter + 1;

        if (newCounter >= BURN_THRESHOLD) {
          // --- COMBUSTION ---
          // The FIRE's current position becomes SAND
          newGrid[y][x] = { type: 'SAND' };
          newColorGrid[y][x] = elements.SAND.color;

          // The SOIL's position becomes FIRE
          newGrid[ny][nx] = { type: 'FIRE', counter: 0 }; // Reset counter
          newColorGrid[ny][nx] = elements.FIRE.color;

          moved[y][x] = true;
          moved[ny][nx] = true;

          return; // Exit after one reaction
        } else {
          // Increment the counter on the SOIL cell
          soilCell.counter = newCounter;
        }
        
        // We found a SOIL to heat up, so we don't need to check other neighbors
        return;
      }
    }
  }
};