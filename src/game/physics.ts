import { type ElementName, type MoveDirection } from "../types/elements";
import { handleSoil } from "./behaviors/soilBehavior";
import { handleWater } from "./behaviors/waterBehavior";
import { handleMud } from "./behaviors/mudBehavior";

// Define the context for behaviors
interface BehaviorContext {
  grid: ElementName[][];
  lastMoveGrid: MoveDirection[][];
  colorGrid: string[][];
  newGrid: ElementName[][];
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
  WATER: handleWater,
  MUD: handleMud,
};

// Simple physics simulation function
export const simulatePhysics = (
  grid: ElementName[][],
  lastMoveGrid: MoveDirection[][],
  colorGrid: string[][]
): {
  newGrid: ElementName[][];
  newLastMoveGrid: MoveDirection[][];
  newColorGrid: string[][];
} => {
  // Create new grids by copying the current state. This is the double-buffering approach.
  // All modifications will be made to these new grids.
  const newGrid = grid.map(row => [...row]);
  const newColorGrid = colorGrid.map(row => [...row]);
  const newLastMoveGrid: MoveDirection[][] = lastMoveGrid.map(row => [...row]); // Copy lastMoveGrid as well

  const height = grid.length;
  const width = grid[0].length;

  // Create a grid to track which cells have already been moved in this simulation step
  const moved = Array(height).fill(null).map(() => Array(width).fill(false));

  // Process grid from bottom to top for gravity simulation
  const xIndices = Array.from(Array(width).keys());

  for (let y = height - 2; y >= 0; y--) {
    // Shuffle x-indices to randomize processing order for the row (Fisher-Yates shuffle)
    for (let i = xIndices.length - 1; i > 0; i--) {
      const j = Math.floor(Math.random() * (i + 1));
      [xIndices[i], xIndices[j]] = [xIndices[j], xIndices[i]];
    }

    for (const x of xIndices) {
      // Skip if this cell has already been moved
      if (moved[y][x]) {
        continue;
      }

      const element = grid[y][x];
      // const color = colorGrid[y][x];

      // Skip empty cells and static elements
      if (element === 'EMPTY') {
        continue;
      }

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

  return { newGrid, newLastMoveGrid, newColorGrid };
};
// Calculate statistics for elements in the grid
export const calculateStats = (grid: ElementName[][]): Record<ElementName, number> => {
  const stats: Record<ElementName, number> = {
    EMPTY: 0, // Still need to initialize for type compatibility
    SOIL: 0,
    WATER: 0,
    FIRE: 0,
    MUD: 0,
    STEAM: 0,
  };
  
  for (let y = 0; y < grid.length; y++) {
    for (let x = 0; x < grid[y].length; x++) {
      const element = grid[y][x];
      if (element in stats && element !== 'EMPTY') {
        stats[element as ElementName]++;
      }
    }
  }
  
  // Don't count EMPTY in stats
  stats.EMPTY = 0;
  
  return stats;
};

// Physics simulation for particles
const GRAVITY = 0.05;
const FRICTION = 0.99;

export const simulateParticles = (particles: import("../types/elements").Particle[], grid: ElementName[][]): import("../types/elements").Particle[] => {
  const height = grid.length;
  const width = grid[0].length;

  const updatedParticles = particles.map(p => {
    // Apply gravity
    p.vy += GRAVITY;

    // Apply friction
    p.vx *= FRICTION;
    p.vy *= FRICTION;

    // Update position
    p.px += p.vx;
    p.py += p.vy;

    // Decrease lifespan
    p.life -= 1;

    // Simple collision with grid boundaries
    if (p.px < 0) {
      p.px = 0;
      p.vx *= -0.5; // Bounce
    }
    if (p.px >= width) {
      p.px = width - 1;
      p.vx *= -0.5; // Bounce
    }
    if (p.py < 0) {
      p.py = 0;
      p.vy *= -0.5; // Bounce
    }
    if (p.py >= height) {
      p.py = height - 1;
      p.vy = 0; // Stop at the bottom
      p.vx = 0;
    }

    // TODO: Add collision with grid cells

    return p;
  });

  // Filter out dead particles
  return updatedParticles.filter(p => p.life > 0);
};