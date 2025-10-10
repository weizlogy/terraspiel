import { type Particle, type Cell, type MoveDirection } from "../../types/elements";
import useGameStore from "../../stores/gameStore";

// This context should match the one in physics.ts
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

const ETHER_SPAWN_CHANCE = 0.01; // Low chance to emit ETHER
const NO_CONSUME_CHANCE = 0.05; // Even lower chance to not consume ETHER on emission

export const handleCrystal = (context: BehaviorContext): Particle | null => {
  const { grid, newGrid, newColorGrid, colorGrid, moved, x, y } = context;
  const elements = useGameStore.getState().elements;
  const cell = grid[y][x];

  let { etherStorage } = cell;

  // Initialize etherStorage if it doesn't exist (e.g., for placed crystals)
  if (etherStorage === undefined) {
    etherStorage = Math.floor(Math.random() * 10) + 5; // Random initial storage
  }

  // Behavior: emitting ETHER
  if (Math.random() < ETHER_SPAWN_CHANCE) {
    // Check if there is ETHER to release
    if (etherStorage > 0) {
      let consumed = true;
      if (Math.random() < NO_CONSUME_CHANCE) {
        consumed = false;
      }

      if (consumed) {
        etherStorage--;
      }

      // If all ether is released, the crystal is destroyed
      if (etherStorage <= 0) {
        newGrid[y][x] = { type: 'EMPTY' };
        newColorGrid[y][x] = elements.EMPTY.color;
        moved[y][x] = true;
        return null; // No particle is emitted upon destruction
      }

      // Update the cell with the new ether storage
      newGrid[y][x] = { ...cell, etherStorage };
      newColorGrid[y][x] = colorGrid[y][x]; // Carry over color

      // Spawn ETHER particle in a random direction
      const angle = Math.random() * 2 * Math.PI;
      const vx = Math.cos(angle) * 0.3;
      const vy = Math.sin(angle) * 0.3;

      const newParticle: Particle = {
        id: -1, // ID will be assigned in simulateWorld
        px: x + 0.5,
        py: y + 0.5,
        vx,
        vy,
        type: 'ETHER',
        life: 150,
      };

      return newParticle;
    }
  }

  // No particle was emitted. Just carry over the state.
  newGrid[y][x] = { ...cell, etherStorage };
  newColorGrid[y][x] = colorGrid[y][x];

  return null;
};
