import { type Particle, type Cell, type ElementName } from "../../types/elements";

interface EtherBehaviorContext {
  particles: Particle[];
  grid: Cell[][];
  width: number;
  height: number;
}

const DEEPENING_CHANCE = 0.15; // Chance for a particle to transform a cell on contact

// Rules for ETHER transforming other elements
const deepeningRules: Partial<Record<ElementName, ElementName>> = {
  SOIL: 'FERTILE_SOIL',
  WATER: 'CLOUD',
  MUD: 'PEAT',
};

export const handleEtherParticles = ({
  particles,
  grid,
  width,
  height,
}: EtherBehaviorContext): { updatedParticles: Particle[], updatedGrid: Cell[][], gridChanged: boolean } => {

  const newGrid = grid.map(row => row.map(cell => ({ ...cell })));
  let gridChanged = false;

  // Filter out dead particles first
  const livingParticles = particles.filter(p => p.life > 0);

  const updatedParticles = livingParticles.map(p => {
    const newParticle = { ...p };

    // We only care about ETHER particles here
    if (newParticle.type !== 'ETHER') {
      return newParticle;
    }

    // 1. Update lifespan
    newParticle.life -= 1;
    if (newParticle.life <= 0) {
      return null; // Will be filtered out later
    }

    // 2. Update velocity for a drifting effect
    newParticle.vx += (Math.random() - 0.5) * 0.15;
    newParticle.vy += (Math.random() - 0.5) * 0.15;

    // Clamp velocity to prevent it from getting too fast
    newParticle.vx = Math.max(-0.5, Math.min(0.5, newParticle.vx));
    newParticle.vy = Math.max(-0.5, Math.min(0.5, newParticle.vy));

    // 3. Update position
    newParticle.px += newParticle.vx;
    newParticle.py += newParticle.vy;

    // 4. Handle boundary collisions
    if (newParticle.px < 0) { newParticle.px = 0; newParticle.vx *= -0.5; }
    if (newParticle.px >= width) { newParticle.px = width - 1; newParticle.vx *= -0.5; }
    if (newParticle.py < 0) { newParticle.py = 0; newParticle.vy *= -0.5; }
    if (newParticle.py >= height) { newParticle.py = height - 1; newParticle.vy *= -0.5; }

    // 5. Handle interaction with the grid (deepening)
    const cx = Math.floor(newParticle.px);
    const cy = Math.floor(newParticle.py);

    if (cx >= 0 && cx < width && cy >= 0 && cy < height) {
      const cellType = newGrid[cy][cx].type;
      const targetType = deepeningRules[cellType];

      // If the particle is over a transformable cell, try to deepen it
      if (targetType && Math.random() < DEEPENING_CHANCE) {
        newGrid[cy][cx] = { type: targetType };
        gridChanged = true;
        newParticle.life = 0; // Consume the particle upon transformation
      }
    }

    return newParticle;
  }).filter(p => p !== null) as Particle[];

  return { updatedParticles, updatedGrid: newGrid, gridChanged };
};