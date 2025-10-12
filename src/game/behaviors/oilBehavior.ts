import { type Cell, type Particle } from "../../types/elements";
import useGameStore from "../../stores/gameStore";

interface BehaviorContext {
  grid: Cell[][];
  newGrid: Cell[][];
  newColorGrid: string[][];
  moved: boolean[][];
  x: number;
  y: number;
  width: number;
  height: number;
}

const SPONTANEOUS_COMBUSTION_CHANCE = 0.001; // 自然発火の確率

export const handleOil = ({
  newGrid,
  x,
  y,
}: BehaviorContext): Particle | void => {
  const elements = useGameStore.getState().elements;
  if (Object.keys(elements).length === 0) return;

  // --- Spontaneous Combustion ---
  if (Math.random() < SPONTANEOUS_COMBUSTION_CHANCE) {
    // Instead of creating a FIRE cell, spawn a FIRE particle
    newGrid[y][x] = { type: 'EMPTY' }; // The oil is consumed
    const fireParticle: Particle = {
      id: -1, // The physics engine will assign a real ID
      type: 'FIRE',
      px: x + 0.5,
      py: y + 0.5,
      vx: (Math.random() - 0.5) * 1.5,
      vy: (Math.random() - 0.5) * 1.5 - 1,
      life: Math.floor(Math.random() * 20) + 40, // Lifespan of 40-60 frames
    };
    return fireParticle;
  }
};
