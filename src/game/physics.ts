import { type ElementName, type MoveDirection, type Cell, type Particle } from "../types/elements";
import { handleSoil } from "./behaviors/soilBehavior";
import { handleWater } from "./behaviors/waterBehavior";
import { handleMud } from "./behaviors/mudBehavior";
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
}

// Define the behavior function type
type ElementBehavior = (context: BehaviorContext) => void;

// Map elements to their behavior handlers
const behaviors: Partial<Record<ElementName, ElementBehavior>> = {
  SOIL: handleSoil,
  FERTILE_SOIL: handleSoil,
  PEAT: handleSoil,
  WATER: handleWater,
  MUD: handleMud,
  CLOUD: handleCloud,
  CLAY: handleSoil,
  FIRE: handleFire,
  SAND: handleMud, // SAND also behaves like mud (fluid)
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
  // Directly modify the write buffers instead of creating copies
  const moved = Array(height).fill(null).map(() => Array(width).fill(false));
  const xIndices = Array.from(Array(width).keys());

  for (let y = height - 1; y >= 0; y--) {
    for (let i = xIndices.length - 1; i > 0; i--) {
      const j = Math.floor(Math.random() * (i + 1));
      [xIndices[i], xIndices[j]] = [xIndices[j], xIndices[i]];
    }

    for (const x of xIndices) {
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
    for (const x of xIndices) {
      const newParticle = handleTransformations({
        grid: writeGrid, // Read from and write to the same grid
        newGrid: writeGrid, // Pass it as newGrid as well
        x, y, width, height,
      });
      if (newParticle) {
        spawnedParticles.push(newParticle);
      }
    }
  }

  // Define elements that should have color variation
  const elementsWithVariation: Array<ElementName> = ['SOIL', 'WATER', 'MUD', 'FERTILE_SOIL', 'PEAT', 'CLAY', 'SAND', 'STONE']; // Add as needed

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
          writeColorGrid[y][x] = elementsWithVariation.includes(newType) ? varyColor(baseColor) : baseColor;
        }
      }
    }
  }

  return { newParticles: updatedParticles };
};

// Calculate statistics for elements in the grid
export const calculateStats = (grid: Cell[][], particles: Particle[]): Record<string, number> => {
  const stats: Record<string, number> = {
    SOIL: 0,
    WATER: 0,
    MUD: 0,
    FERTILE_SOIL: 0,
    PEAT: 0,
    CLOUD: 0,
    CLAY: 0,
    FIRE: 0,
    SAND: 0,
    STONE: 0,
    ETHER: 0,
  };

  for (let y = 0; y < grid.length; y++) {
    for (let x = 0; x < grid[y].length; x++) {
      const element = grid[y][x].type;
      if (element in stats) {
        stats[element]++;
      }
    }
  }

  for (const particle of particles) {
    if (particle.type === 'ETHER') {
      stats.ETHER++;
    }
  }

  return stats;
};