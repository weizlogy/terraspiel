import { type Particle, type Cell, type ElementName } from "../../types/elements";
import useGameStore from "../../stores/gameStore";

interface ThunderBehaviorContext {
  particles: Particle[];
  grid: Cell[][];
  width: number;
  height: number;
  spawnedParticles: Particle[];
  nextParticleId: number;
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

// Elements that can be turned into particles by an explosion
const AFFECTED_BY_EXPLOSION: ElementName[] = ['SOIL', 'SAND', 'WATER', 'MUD', 'PEAT', 'FERTILE_SOIL', 'CLAY', 'FIRE', 'PLANT', 'SEED', 'OIL'];

const createExplosion = (
  grid: Cell[][],
  cx: number, 
  cy: number, 
  radius: number, 
  width: number, 
  height: number, 
  spawnedParticles: Particle[], 
  nextParticleId: number
): number => {
  let currentParticleId = nextParticleId;
  for (let j = -radius; j <= radius; j++) {
    for (let i = -radius; i <= radius; i++) {
      const nx = cx + i;
      const ny = cy + j;

      if (nx >= 0 && nx < width && ny >= 0 && ny < height) {
        const distance = Math.sqrt(i * i + j * j);
        if (distance <= radius) {
          const cellType = grid[ny][nx].type;
          
          if (AFFECTED_BY_EXPLOSION.includes(cellType)) {
            const probability = 1.0 - (distance / radius);
            if (Math.random() < probability) {
              const power = (1.0 - (distance / radius)) * 2.0; // Explosion power decreases with distance
              const particle: Particle = {
                id: currentParticleId++,
                type: cellType,
                px: nx + 0.5,
                py: ny + 0.5,
                vx: i * power * 0.5, // Directional velocity
                vy: j * power * 0.5,
                life: 100, // Give it some life to fall back down
              };
              spawnedParticles.push(particle);
              grid[ny][nx] = { type: 'EMPTY' }; // Clear the original cell
            }
          }
        }
      }
    }
  }
  return currentParticleId;
};

export const handleThunderParticles = ({
  particles,
  grid,
  width,
  height,
  spawnedParticles,
  nextParticleId,
}: ThunderBehaviorContext): { updatedParticles: Particle[], updatedGrid: Cell[][], gridChanged: boolean, nextParticleId: number } => {
  const elements = useGameStore.getState().elements;
  const newGrid = grid.map(row => row.map(cell => ({ ...cell })));
  let gridChanged = false;
  let currentParticleId = nextParticleId;

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
        const radius = Math.floor(Math.random() * 2) + 1; // Random radius 1-2
        currentParticleId = createExplosion(newGrid, cx, cy, radius, width, height, spawnedParticles, currentParticleId);
        gridChanged = true;
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
        const radius = Math.floor(Math.random() * 3) + 1; // Random radius 1-3
        currentParticleId = createExplosion(newGrid, cx, cy, radius, width, height, spawnedParticles, currentParticleId);
        gridChanged = true;
        newParticle.life = 0; // Consume the particle upon transformation
      }
    }

    return newParticle;
  }).filter(p => p !== null) as Particle[];

  return { updatedParticles, updatedGrid: newGrid, gridChanged, nextParticleId: currentParticleId };
};