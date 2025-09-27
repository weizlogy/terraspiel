import { type ElementName, type MoveDirection, type Cell, type Particle } from "../types/elements";
import { handleSoil } from "./behaviors/soilBehavior";
import { handleWater } from "./behaviors/waterBehavior";
import { handleMud } from "./behaviors/mudBehavior";
import { handleCloud } from "./behaviors/cloudBehavior";
import { handleTransformations } from "./transformation";
import { handleEtherParticles } from "./behaviors/etherBehavior";
import useGameStore from "../stores/gameStore";

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
};

// Main physics simulation function that handles cells and particles
export const simulateWorld = (
  grid: Cell[][],
  lastMoveGrid: MoveDirection[][],
  colorGrid: string[][],
  particles: Particle[],
): {
  newGrid: Cell[][];
  newLastMoveGrid: MoveDirection[][];
  newColorGrid: string[][];
  newParticles: Particle[];
} => {
  // Add a guard clause to check if the grid is initialized
  if (!grid || grid.length === 0 || !grid[0] || grid[0].length === 0) {
    // Grid not initialized yet, return current state
    return { newGrid: grid, newLastMoveGrid: lastMoveGrid, newColorGrid: colorGrid, newParticles: particles };
  }

  const height = grid.length;
  const width = grid[0].length;
  const elements = useGameStore.getState().elements;

  if (Object.keys(elements).length === 0) {
    // Elements not loaded yet, return current state
    return { newGrid: grid, newLastMoveGrid: lastMoveGrid, newColorGrid: colorGrid, newParticles: particles };
  }

  // --- PASS 1: MOVEMENT ---
  const newGrid = grid.map(row => row.map(cell => ({ ...cell })));
  const newColorGrid = colorGrid.map(row => [...row]);
  const newLastMoveGrid: MoveDirection[][] = lastMoveGrid.map(row => [...row]);
  const moved = Array(height).fill(null).map(() => Array(width).fill(false));
  const xIndices = Array.from(Array(width).keys());

  for (let y = height - 2; y >= 0; y--) {
    for (let i = xIndices.length - 1; i > 0; i--) {
      const j = Math.floor(Math.random() * (i + 1));
      [xIndices[i], xIndices[j]] = [xIndices[j], xIndices[i]];
    }

    for (const x of xIndices) {
      if (moved[y][x]) continue;

      const element = grid[y][x].type;
      if (element === 'EMPTY') continue;

      const behavior = behaviors[element];
      if (behavior) {
        behavior({
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
        });
      }
    }
  }

  // --- PASS 2: TRANSFORMATIONS ---
  const gridAfterMove = newGrid.map(row => row.map(cell => ({ ...cell })));
  const spawnedParticles: Particle[] = [];

  for (let y = height - 1; y >= 0; y--) {
    for (const x of xIndices) {
      const newParticle = handleTransformations({
        grid: newGrid, // Read from the result of the physics pass
        newGrid: gridAfterMove, // Write to the grid for this pass
        x, y, width, height,
      });
      if (newParticle) {
        spawnedParticles.push(newParticle);
      }
    }
  }

  // Update color grid after transformations
  for (let y = 0; y < height; y++) {
    for (let x = 0; x < width; x++) {
      if (newGrid[y][x].type !== gridAfterMove[y][x].type) {
        const newType = gridAfterMove[y][x].type;
        newColorGrid[y][x] = elements[newType]?.color || '#000000';
      }
    }
  }

  // --- PASS 3: PARTICLE SIMULATION & DEEPENING ---
  const allParticles = particles.concat(spawnedParticles);
  const { updatedParticles, updatedGrid, gridChanged } = handleEtherParticles({
    particles: allParticles,
    grid: gridAfterMove,
    width,
    height,
  });

  // If particles changed the grid, we need to update the color grid accordingly
  if (gridChanged) {
    for (let y = 0; y < height; y++) {
      for (let x = 0; x < width; x++) {
        if (gridAfterMove[y][x].type !== updatedGrid[y][x].type) {
          const newType = updatedGrid[y][x].type;
          newColorGrid[y][x] = elements[newType]?.color || '#000000';
        }
      }
    }
  }

  return { newGrid: updatedGrid, newLastMoveGrid, newColorGrid, newParticles: updatedParticles };
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