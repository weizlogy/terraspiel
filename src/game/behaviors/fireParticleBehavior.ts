import { type Particle, type Cell, type ElementName } from "../../types/elements";
import useGameStore from "../../stores/gameStore";

interface FireParticleBehaviorContext {
  particles: Particle[];
  grid: Cell[][];
  width: number;
  height: number;
  nextParticleId: number;
}

const BURN_CHANCE = 0.15; // Chance for a fire particle to ignite a neighbor without moving
const SPREAD_CHANCE = 0.65; // Chance to spread to a neighbor when its life ends

// Rules for what FIRE transforms other elements into
const fireTransformationRules: Partial<Record<ElementName, ElementName>> = {
  SOIL: 'SAND',
  CLAY: 'STONE',
  STONE: 'MAGMA',
  PLANT: 'FIRE',
  OIL: 'FIRE',
  PEAT: 'FIRE',
  FERTILE_SOIL: 'FIRE',
  SAND: 'MAGMA',
};

export const handleFireParticles = ({
  particles,
  grid,
  width,
  height,
  nextParticleId,
}: FireParticleBehaviorContext): { updatedParticles: Particle[], updatedGrid: Cell[][], gridChanged: boolean, nextParticleId: number } => {
  const newGrid = grid.map(row => row.map(cell => ({ ...cell })));
  let gridChanged = false;
  let currentParticleId = nextParticleId;

  const fireParticles = particles.filter(p => p.type === 'FIRE');
  const otherParticles = particles.filter(p => p.type !== 'FIRE');

  const newlySpawnedParticles: Particle[] = [];

  const updatedFireParticles = fireParticles.map(p => {
    const newParticle = { ...p };
    newParticle.life--; // Decrease lifespan each frame

    const cx = Math.floor(newParticle.px);
    const cy = Math.floor(newParticle.py);

    // 1a. Check for contact with CRYSTAL, which transforms it and extinguishes the fire
    for (let j = -1; j <= 1; j++) {
      for (let i = -1; i <= 1; i++) {
        if (i === 0 && j === 0) continue;
        const nx = cx + i;
        const ny = cy + j;
        if (nx >= 0 && nx < width && ny >= 0 && ny < height && newGrid[ny][nx].type === 'CRYSTAL') {
          newGrid[ny][nx] = { type: 'RUBY' }; // Transform CRYSTAL to RUBY
          gridChanged = true;
          newParticle.life = 0; // Extinguish the fire particle
          break;
        }
      }
      if (newParticle.life <= 0) break;
    }

    // If extinguished by touching crystal, filter it out immediately
    if (newParticle.life <= 0) {
      return null;
    }

    // 1b. Check for immediate extinguishment by water
    for (let j = -1; j <= 1; j++) {
      for (let i = -1; i <= 1; i++) {
        const nx = cx + i;
        const ny = cy + j;
        if (nx >= 0 && nx < width && ny >= 0 && ny < height && newGrid[ny][nx].type === 'WATER') {
          newParticle.life = 0; // Extinguish immediately
          break;
        }
      }
      if (newParticle.life <= 0) break;
    }

    // 2. If life has run out, try to burn the current cell and spread
    if (newParticle.life <= 0) {
      const currentCell = newGrid[cy][cx];
      const transformationResult = fireTransformationRules[currentCell.type];

      // Burn the cell the fire is on
      if (transformationResult) {
        newGrid[cy][cx] = { type: transformationResult === 'FIRE' ? 'EMPTY' : transformationResult };
        gridChanged = true;
      }

      // Scan for flammable neighbors to spread to
      const flammableNeighbors: { x: number, y: number }[] = [];
      for (let j = -1; j <= 1; j++) {
        for (let i = -1; i <= 1; i++) {
          if (i === 0 && j === 0) continue;
          const nx = cx + i;
          const ny = cy + j;
          if (nx >= 0 && nx < width && ny >= 0 && ny < height && fireTransformationRules[newGrid[ny][nx].type]) {
            flammableNeighbors.push({ x: nx, y: ny });
          }
        }
      }

      // Try to spread to a random flammable neighbor
      if (flammableNeighbors.length > 0 && Math.random() < SPREAD_CHANCE) {
        const target = flammableNeighbors[Math.floor(Math.random() * flammableNeighbors.length)];
        newlySpawnedParticles.push({
          id: currentParticleId++,
          type: 'FIRE',
          px: target.x + 0.5,
          py: target.y + 0.5,
          vx: 0,
          vy: 0,
          life: Math.floor(Math.random() * 40) + 80, // New fire gets a lifespan
        });
      }
      
      return null; // Mark for removal
    }

    // 3. During its lifespan, the fire has a small chance to ignite adjacent flammable materials without moving
    if (Math.random() < BURN_CHANCE) {
        const flammableNeighbors: { x: number, y: number, type: ElementName }[] = [];
        for (let j = -1; j <= 1; j++) {
            for (let i = -1; i <= 1; i++) {
                if (i === 0 && j === 0) continue;
                const nx = cx + i;
                const ny = cy + j;
                if (nx >= 0 && nx < width && ny >= 0 && ny < height) {
                    const neighborType = newGrid[ny][nx].type;
                    if (fireTransformationRules[neighborType]) {
                        flammableNeighbors.push({ x: nx, y: ny, type: neighborType });
                    }
                }
            }
        }

        if (flammableNeighbors.length > 0) {
            const target = flammableNeighbors[Math.floor(Math.random() * flammableNeighbors.length)];
            const newType = fireTransformationRules[target.type];

            if (newType) {
                if (newType === 'FIRE') {
                    // If the target becomes fire, spawn a new particle there
                    newGrid[target.y][target.x] = { type: 'EMPTY' };
                    newlySpawnedParticles.push({
                        id: currentParticleId++,
                        type: 'FIRE',
                        px: target.x + 0.5,
                        py: target.y + 0.5,
                        vx: 0,
                        vy: 0,
                        life: Math.floor(Math.random() * 40) + 80,
                    });
                } else {
                    // Otherwise, just transform the cell
                    newGrid[target.y][target.x] = { type: newType };
                }
                gridChanged = true;
            }
        }
    }

    return newParticle;

  }).filter((p): p is Particle => p !== null);

  const updatedParticles = [...otherParticles, ...updatedFireParticles, ...newlySpawnedParticles];

  return { updatedParticles, updatedGrid: newGrid, gridChanged, nextParticleId: currentParticleId };
};
