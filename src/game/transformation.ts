import { type Cell, type ElementName } from "../types/elements";
import { transformationRules } from "./rules";

interface BehaviorContext {
  grid: Cell[][];
  newGrid: Cell[][];
  x: number;
  y: number;
  width: number;
  height: number;
}

// Helper function to get the type of a cell at a given coordinate
const getCellType = (grid: Cell[][], x: number, y: number, width: number, height: number): ElementName | null => {
  if (x >= 0 && x < width && y >= 0 && y < height) {
    return grid[y][x].type;
  }
  return null;
};

export const handleTransformations = ({
  grid,
  newGrid,
  x,
  y,
  width,
  height,
}: BehaviorContext): void => {
  const cell = grid[y][x];
  const applicableRules = transformationRules.filter(rule => rule.from === cell.type);

  if (applicableRules.length === 0) {
    return;
  }

  // Count neighbors
  const neighborCounts: Partial<Record<ElementName, number>> = {};
  for (let j = -1; j <= 1; j++) {
    for (let i = -1; i <= 1; i++) {
      if (i === 0 && j === 0) continue;
      const neighborType = getCellType(grid, x + i, y + j, width, height);
      if (neighborType) {
        neighborCounts[neighborType] = (neighborCounts[neighborType] || 0) + 1;
      }
    }
  }

  // Check each applicable rule
  for (const rule of applicableRules) {
    let conditionsMet = true;
    for (const condition of rule.conditions.surrounding) {
      const count = neighborCounts[condition.type] || 0;
      if (count < (condition.min || 0) || count > (condition.max || 8)) {
        conditionsMet = false;
        break;
      }
    }

    if (conditionsMet) {
      // Conditions are met, process the transformation probability
      if (Math.random() < rule.probability) {
        const currentCounter = newGrid[y][x].counter || 0;
        const newCounter = currentCounter + 1;

        if (newCounter >= rule.threshold) {
          // Transform the cell
          newGrid[y][x].type = rule.to;
          newGrid[y][x].counter = 0; // Reset counter after transformation
        } else {
          // Increment counter
          newGrid[y][x].counter = newCounter;
        }
      }
      // If we found a matching rule and processed it, we can stop.
      // This prevents multiple transformations in one step.
      break;
    }
  }
};
