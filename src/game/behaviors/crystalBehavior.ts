import { type Particle, type Cell, type MoveDirection } from "../../types/elements";

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

const ETHER_SPAWN_CHANCE = 0.001; // 微弱に放出する

export const handleCrystal = (context: BehaviorContext): Particle | null => {
  const { x, y } = context;

  // Behavior: emitting ETHER
  if (Math.random() < ETHER_SPAWN_CHANCE) {
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

  return null;
};
