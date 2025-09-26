import { type ElementName, type MoveDirection, type Cell } from "../types/elements";
import { handleSoil } from "./behaviors/soilBehavior";
import { handleWater } from "./behaviors/waterBehavior";
import { handleMud } from "./behaviors/mudBehavior";
import { handleTransformations } from "./transformation";

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
  WATER: handleWater,
  MUD: handleMud,
};

// Simple physics simulation function
export const simulatePhysics = (
  grid: Cell[][],
  lastMoveGrid: MoveDirection[][],
  colorGrid: string[][]
): {
  newGrid: Cell[][];
  newLastMoveGrid: MoveDirection[][];
  newColorGrid: string[][];
} => {
  // --- PASS 1: MOVEMENT ---
  const newGrid = grid.map(row => row.map(cell => ({ ...cell })));
  const newColorGrid = colorGrid.map(row => [...row]);
  const newLastMoveGrid: MoveDirection[][] = lastMoveGrid.map(row => [...row]);

  const height = grid.length;
  const width = grid[0].length;

  const moved = Array(height).fill(null).map(() => Array(width).fill(false));

  const xIndices = Array.from(Array(width).keys());

  for (let y = height - 2; y >= 0; y--) {
    for (let i = xIndices.length - 1; i > 0; i--) {
      const j = Math.floor(Math.random() * (i + 1));
      [xIndices[i], xIndices[j]] = [xIndices[j], xIndices[i]];
    }

    for (const x of xIndices) {
      if (moved[y][x]) {
        continue;
      }

      const element = grid[y][x].type;
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

  // --- PASS 2: TRANSFORMATIONS ---
  const finalGrid = newGrid.map(row => row.map(cell => ({ ...cell })));

  for (let y = height - 1; y >= 0; y--) {
    for (const x of xIndices) {
      handleTransformations({
        grid: newGrid, // Read from the result of the physics pass
        newGrid: finalGrid, // Write to the final grid
        x,
        y,
        width,
        height,
      });
    }
  }

  return { newGrid: finalGrid, newLastMoveGrid, newColorGrid };
};

// Calculate statistics for elements in the grid
export const calculateStats = (grid: Cell[][]): Record<ElementName, number> => {
  const stats: Record<ElementName, number> = {
    EMPTY: 0,
    SOIL: 0,
    WATER: 0,
    FIRE: 0,
    MUD: 0,
    STEAM: 0,
  };
  
  for (let y = 0; y < grid.length; y++) {
    for (let x = 0; x < grid[y].length; x++) {
      const element = grid[y][x].type;
      if (element in stats && element !== 'EMPTY') {
        stats[element as ElementName]++;
      }
    }
  }
  
  stats.EMPTY = 0;
  
  return stats;
};

// Physics simulation for particles
const GRAVITY = 0.05;
const FRICTION = 0.99;

export const simulateParticles = (particles: import("../types/elements").Particle[], grid: Cell[][]): import("../types/elements").Particle[] => {
  const height = grid.length;
  const width = grid[0].length;

  const updatedParticles = particles.map(p => {
    p.vy += GRAVITY;
    p.vx *= FRICTION;
    p.py += p.vy;
    p.life -= 1;

    if (p.px < 0) { p.px = 0; p.vx *= -0.5; }
    if (p.px >= width) { p.px = width - 1; p.vx *= -0.5; }
    if (p.py < 0) { p.py = 0; p.vy *= -0.5; }
    if (p.py >= height) { p.py = height - 1; p.vy = 0; p.vx = 0; }

    return p;
  });

  return updatedParticles.filter(p => p.life > 0);
};