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
    const { grid, x, y, height } = context;
    const cell = grid[y][x];

    if (cell.plantMode === 'withered') {
        handleGranular(context);
        return;
    }

    // Check if the cell below is empty
    if (y + 1 < height && grid[y + 1][x].type === 'EMPTY') {
        handleGranular(context);
    } else {
        // Other plant modes are static if not falling
        const { newGrid, newColorGrid, colorGrid } = context;
        newGrid[y][x] = cell;
        newColorGrid[y][x] = colorGrid[y][x];
    }
};