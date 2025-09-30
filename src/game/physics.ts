import { type ElementName, type MoveDirection, type Cell, type Particle } from "../types/elements";
import { handleGranular } from "./behaviors/granularBehavior";
import { handleCloud } from "./behaviors/cloudBehavior";
import { handleFire } from "./behaviors/fireBehavior";
import { handleTransformations } from "./transformation";
import { handleEtherParticles } from "./behaviors/etherBehavior";
import useGameStore from "../stores/gameStore";
import { varyColor } from "../utils/colors";

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

// Define the behavior function type
type ElementBehavior = (context: BehaviorContext) => void;

// Map elements to their behavior handlers
const behaviors: Partial<Record<ElementName, ElementBehavior>> = {
  SOIL: handleGranular,
  FERTILE_SOIL: handleGranular,
  PEAT: handleGranular,
  WATER: handleGranular,
  MUD: handleGranular,
  CLOUD: handleCloud,
  CLAY: handleGranular,
  FIRE: handleFire,
  SAND: handleGranular,
  STONE: handleGranular, // Will be handled by the guard clause in handleGranular
};

// Main physics simulation function that handles cells and particles
export const simulateWorld = (
  readGrid: Cell[][],
  readLastMoveGrid: MoveDirection[][],
  readColorGrid: string[][],
  writeGrid: Cell[][],
  writeLastMoveGrid: MoveDirection[][],
  writeColorGrid: string[][],
  particles: Particle[],
  frameCount: number,
): {
  newParticles: Particle[];
} => {
  // Add a guard clause to check if the grid is initialized
  if (!readGrid || readGrid.length === 0 || !readGrid[0] || readGrid[0].length === 0) {
    // Grid not initialized yet, return current state
    return { newParticles: particles };
  }

  const height = readGrid.length;
  const width = readGrid[0].length;
  const elements = useGameStore.getState().elements;

  if (Object.keys(elements).length === 0) {
    // Elements not loaded yet, return current state
    return { newParticles: particles };
  }

  // --- PASS 1: MOVEMENT ---
  const moved = Array(height).fill(null).map(() => Array(width).fill(false));
  const scanRight = frameCount % 2 === 0;

  for (let y = height - 1; y >= 0; y--) {
    for (let i = 0; i < width; i++) {
      const x = scanRight ? i : width - 1 - i;

      if (moved[y][x]) continue;

      const element = readGrid[y][x].type;
      if (element === 'EMPTY') {
        writeGrid[y][x] = { type: 'EMPTY' };
        writeColorGrid[y][x] = elements.EMPTY.color;
        continue;
      }

      const behavior = behaviors[element];
      if (behavior) {
        behavior({
          grid: readGrid, // Pass read-only grid
          lastMoveGrid: readLastMoveGrid,
          colorGrid: readColorGrid,
          newGrid: writeGrid, // Pass writable grid
          newLastMoveGrid: writeLastMoveGrid,
          newColorGrid: writeColorGrid,
          moved,
          x,
          y,
          width,
          height,
          scanRight,
        });
      } else {
        // If no behavior, the element stays in place
        writeGrid[y][x] = readGrid[y][x];
        writeColorGrid[y][x] = readColorGrid[y][x];
      }
    }
  }

  // --- PASS 2: TRANSFORMATIONS ---
  const spawnedParticles: Particle[] = [];

  for (let y = height - 1; y >= 0; y--) {
    for (let i = 0; i < width; i++) {
      const x = scanRight ? i : width - 1 - i;
      const newParticle = handleTransformations({
        grid: writeGrid, // Read from and write to the same grid
        newGrid: writeGrid, // Pass it as newGrid as well
        newColorGrid: writeColorGrid,
        x, y, width, height,
      });
      if (newParticle) {
        spawnedParticles.push(newParticle);
      }
    }
  }

  // Color variation is now handled by the hasColorVariation property in elements.json

  // Update color grid after transformations
  // This loop is now more complex because we don't have a separate `gridAfterMove`
  // We can skip this color update for now, as the main color update will happen in the behavior itself.
  // This is a potential area for further optimization.

  // --- PASS 3: PARTICLE SIMULATION & DEEPENING ---
  const allParticles = particles.concat(spawnedParticles);
  const { updatedParticles, updatedGrid, gridChanged } = handleEtherParticles({
    particles: allParticles,
    grid: writeGrid, // Use the latest grid state
    width,
    height,
  });

  // If particles changed the grid, we need to copy the changes back to writeGrid
  if (gridChanged) {
    for (let y = 0; y < height; y++) {
      for (let x = 0; x < width; x++) {
        if (writeGrid[y][x].type !== updatedGrid[y][x].type) {
          writeGrid[y][x] = updatedGrid[y][x];
          const newType = updatedGrid[y][x].type;
          const baseColor = elements[newType]?.color || '#000000';
          writeColorGrid[y][x] = elements[newType]?.hasColorVariation ? varyColor(baseColor) : baseColor;
        }
      }
    }
  }

  return { newParticles: updatedParticles };
};

// Calculate statistics for elements in the grid
export const calculateStats = (grid: Cell[][], particles: Particle[]): Record<ElementName | "ETHER", number> => {
  // Get the current stats object from the store to ensure all elements are present
  const stats = useGameStore.getState().stats;

  // Reset all counts to zero
  for (const key in stats) {
    stats[key as keyof typeof stats] = 0;
  }

  // Recalculate counts from the grid
  for (let y = 0; y < grid.length; y++) {
    for (let x = 0; x < grid[y].length; x++) {
      const element = grid[y][x].type;
      if (element in stats && element !== 'EMPTY') { // Do not count EMPTY
        stats[element]++;
      }
    }
  }

  // Recalculate counts from particles
  for (const particle of particles) {
    if (particle.type in stats) {
      stats[particle.type]++;
    }
  }

  return stats;
};