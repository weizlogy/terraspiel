import { type Particle, type Cell, type ElementName } from "../../types/elements";
import useGameStore from "../../stores/gameStore";

interface ThunderBehaviorContext {
  particles: Particle[];
  grid: Cell[][];
  width: number;
  height: number;
}

const ZAP_CHANCE = 0.5; // Chance for a particle to transform a cell on contact

// Rules for THUNDER transforming other elements
const zapRules: Partial<Record<ElementName, ElementName>> = {
  PLANT: 'FIRE',
  OIL: 'FIRE',
  PEAT: 'FIRE',
  FERTILE_SOIL: 'FIRE',
  SOIL: 'FIRE',
};

export const handleThunderParticles = ({
  particles,
  grid,
  width,
  height,
}: ThunderBehaviorContext): { updatedParticles: Particle[], updatedGrid: Cell[][], gridChanged: boolean } => {
  const elements = useGameStore.getState().elements;
  const newGrid = grid.map(row => row.map(cell => ({ ...cell })));
  let gridChanged = false;

  // Filter out dead particles first
  const livingParticles = particles.filter(p => p.life > 0);

  const updatedParticles = livingParticles.map(p => {
    const newParticle = { ...p };

    // We only care about THUNDER particles here
    if (newParticle.type !== 'THUNDER') {
      return newParticle;
    }

    // 1. Update lifespan
    newParticle.life -= 1;
    if (newParticle.life <= 0) {
      return null; // Will be filtered out later
    }

    // 2. Update velocity for a downward zig-zag effect
    newParticle.vx += (Math.random() - 0.5) * 1.5;
    newParticle.vy += 0.1; // Gravity

    // Clamp velocity
    newParticle.vx = Math.max(-2, Math.min(2, newParticle.vx));
    newParticle.vy = Math.max(-1, Math.min(4, newParticle.vy));

    // 3. Update position
    newParticle.px += newParticle.vx;
    newParticle.py += newParticle.vy;

    // 4. Handle boundary collisions
    if (newParticle.px < 0 || newParticle.px >= width || newParticle.py < 0 || newParticle.py >= height) {
      newParticle.life = 0;
      return null;
    }

    // 5. Handle interaction with the grid (zapping)
    const cx = Math.floor(newParticle.px);
    const cy = Math.floor(newParticle.py);

    if (cx >= 0 && cx < width && cy >= 0 && cy < height) {
      const cellType = newGrid[cy][cx].type;

      // Disappear in water
      if (cellType === 'WATER') {
        newParticle.life = 0;
        return null;
      }

      const targetType = zapRules[cellType];
      
      // Ensure the element type exists in the elements map
      if (!elements[cellType]) {
        return newParticle;
      }

      const element = elements[cellType];

      // If the particle is over a flammable cell, try to ignite it
      if (targetType && element?.isFlammable && Math.random() < ZAP_CHANCE) {
        newGrid[cy][cx] = { type: targetType };
        gridChanged = true;
        newParticle.life = 0; // Consume the particle upon transformation
      }
    }

    return newParticle;
  }).filter(p => p !== null) as Particle[];

  return { updatedParticles, updatedGrid: newGrid, gridChanged };
};