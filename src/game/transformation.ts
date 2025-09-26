import { type Cell, type ElementName } from "../types/elements";
import { transformationRules } from "./rules";
import useGameStore from "../stores/gameStore";

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
          // Consume a neighbor if specified by the rule
          if (rule.consumes) {
            let consumed = false;
            // Find and consume a neighbor (randomize search order)
            const directions = [-1, 0, 1];
            directions.sort(() => Math.random() - 0.5); // Shuffle directions
            for (const j of directions) {
              for (const i of directions) {
                if (i === 0 && j === 0) continue;
                const nx = x + i;
                const ny = y + j;
                if (nx >= 0 && nx < width && ny >= 0 && ny < height) {
                  // Check the original grid for the element to consume
                  if (grid[ny][nx].type === rule.consumes) {
                    // Consume it in the new grid
                    newGrid[ny][nx] = { type: 'EMPTY' };
                    consumed = true;
                    break;
                  }
                }
              }
              if (consumed) break;
            }
          }

          const fromType = newGrid[y][x].type;
          // Transform the cell
          newGrid[y][x].type = rule.to;
          newGrid[y][x].counter = 0; // Reset counter after transformation

          // Spawn ETHER on transformation
          const ETHER_SPAWN_CHANCE = 0.005; // 0.5% chance
          if (fromType !== rule.to && Math.random() < ETHER_SPAWN_CHANCE) {
            const { addParticle } = useGameStore.getState();
            const vx = (Math.random() - 0.5) * 0.3; // Slow, random drift
            const vy = (Math.random() - 0.5) * 0.3;
            addParticle(x + 0.5, y + 0.5, 'ETHER', vx, vy);
          }

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
