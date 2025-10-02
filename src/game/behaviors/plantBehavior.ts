import { type MoveDirection, type Cell } from "../../types/elements";
import { handleGranular } from "./granularBehavior";

// Define the context for behaviors
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
  scanRight: boolean;
}

export const handlePlant = (context: BehaviorContext) => {
    const { grid, x, y } = context;
    const cell = grid[y][x];

    if (cell.plantMode === 'withered') {
        handleGranular(context);
    } else {
        // Other plant modes are static
        const { newGrid, newColorGrid, colorGrid } = context;
        newGrid[y][x] = cell;
        newColorGrid[y][x] = colorGrid[y][x];
    }
};