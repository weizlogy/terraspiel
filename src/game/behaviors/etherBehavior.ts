import { type Particle, type Cell, type ElementName } from "../../types/elements";

// Define the structure of a particle interaction rule
export interface ParticleInteractionRule {
  type: 'particle_interaction';
  particle: ElementName;
  from: ElementName;
  to: ElementName;
  probability: number;
}

interface EtherBehaviorContext {
  particles: Particle[];
  grid: Cell[][];
  width: number;
  height: number;
  rules: ParticleInteractionRule[]; // Pass rules as an argument
}

export const handleEtherParticles = ({
  particles,
  grid,
  width,
  height,
  rules, // Get rules from context
}: EtherBehaviorContext): { updatedParticles: Particle[], updatedGrid: Cell[][], gridChanged: boolean } => {

  const newGrid = grid.map(row => row.map(cell => ({ ...cell })));
  let gridChanged = false;

  // Create a map for quick rule lookup
  const etherRules = new Map<ElementName, { to: ElementName; probability: number }>();
  rules.forEach(rule => {
    if (rule.particle === 'ETHER') {
      etherRules.set(rule.from, { to: rule.to, probability: rule.probability });
    }
  });

  // Filter out dead particles first
  const livingParticles = particles.filter(p => p.life > 0);

  // Build a spatial hash grid for efficient neighbor lookup
  const particleGrid = new Map<string, Particle[]>();
  for (const p of livingParticles) {
    if (p.type === 'ETHER') { // We only need to grid ETHER particles
      const key = `${Math.floor(p.px)},${Math.floor(p.py)}`;
      if (!particleGrid.has(key)) {
        particleGrid.set(key, []);
      }
      particleGrid.get(key)!.push(p);
    }
  }

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
      const rule = etherRules.get(cellType);

      // If the particle is over a transformable cell, try to deepen it
      if (rule && Math.random() < rule.probability) {
        if (rule.to === 'CRYSTAL') {
          let etherStorage = 1; // Start with the current particle's ether

          // Efficiently check for and consume neighboring ETHER particles
          for (let j = -1; j <= 1; j++) {
            for (let i = -1; i <= 1; i++) {
              if (i === 0 && j === 0) continue;
              const key = `${cx + i},${cy + j}`;
              if (particleGrid.has(key)) {
                const neighbors = particleGrid.get(key)!;
                for (const neighbor of neighbors) {
                  // Ensure we don't consume the particle that is causing the transformation
                  if (neighbor.id !== newParticle.id) {
                    etherStorage++;
                    neighbor.life = 0; // Consume the nearby particle
                  }
                }
              }
            }
          }

          newGrid[cy][cx] = { type: 'CRYSTAL', etherStorage };
        } else {
          newGrid[cy][cx] = { type: rule.to };
        }
        
        gridChanged = true;
        newParticle.life = 0; // Consume the particle upon transformation
      }
    }

    return newParticle;
  }).filter(p => p !== null) as Particle[];

  return { updatedParticles, updatedGrid: newGrid, gridChanged };
};
