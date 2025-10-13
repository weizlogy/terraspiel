import { type ElementName, type MoveDirection, type Cell, type Particle } from "../types/elements";
import { handleGranular } from "./behaviors/granularBehavior";
import { handleCloud } from "./behaviors/cloudBehavior";

import { handleTransformations } from "./transformation";
import { handlePlantGrowth } from "./behaviors/plantGrowthBehavior";
import { handlePlant } from "./behaviors/plantBehavior";
import { handleEtherParticles } from "./behaviors/etherBehavior";
import { handleThunderParticles } from "./behaviors/thunderBehavior";
import { handleFireParticles } from "./behaviors/fireParticleBehavior";
import { handleOil } from "./behaviors/oilBehavior";
import { handleCrystal } from "./behaviors/crystalBehavior";
import useGameStore from "../stores/gameStore";
import { varyColor } from "../utils/colors";

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
  scanRight: boolean;
  isChained?: boolean; // Flag for composing behaviors
}

// Define the behavior function type
type ElementBehavior = (context: BehaviorContext) => void | Particle | null;

const handleOilBehavior: ElementBehavior = (context) => {
  const newParticle = handleOil(context);
  if (newParticle) {
    return newParticle;
  }

  // If the cell hasn't been moved by handleOil, apply granular behavior
  if (!context.moved[context.y][context.x]) {
    handleGranular(context);
  }
};

const handleCrystalBehavior: ElementBehavior = (context) => {
  // 1. Run crystal's unique logic first.
  const spawnedParticle = handleCrystal(context);

  // 2. If the crystal was destroyed, we're done.
  if (context.newGrid[context.y][context.x].type === 'EMPTY') {
    return spawnedParticle;
  }

  // 3. Apply granular logic, chaining it after the crystal logic.
  context.isChained = true;
  handleGranular(context);
  delete context.isChained; // Clean up the flag

  return spawnedParticle;
};

// Map elements to their behavior handlers
const behaviors: Partial<Record<ElementName, ElementBehavior>> = {
  SOIL: handleGranular,
  FERTILE_SOIL: handleGranular,
  PEAT: handleGranular,
  WATER: handleGranular,
  MUD: handleGranular,
  CLOUD: handleCloud,
  CLAY: handleGranular,
  SAND: handleGranular,
  STONE: handleGranular, // Will be handled by the guard clause in handleGranular
  BASALT: handleGranular,
  OBSIDIAN: handleGranular,
  SEED: handleGranular,
  OIL: handleOilBehavior,
  PLANT: handlePlant,
  CRYSTAL: handleCrystalBehavior,
  ELECTRUM: handleGranular,
  RUBY: handleGranular,
  SAPPHIRE: handleGranular,
  AMETHYST: handleGranular,
  GARNET: handleGranular,
  EMERALD: handleGranular,
  MAGMA: handleGranular,
};

// Main physics simulation function that handles cells and particles
export const simulateWorld = (
  readGrid: Cell[][],
  readLastMoveGrid: MoveDirection[][],
  readColorGrid: string[][],
  writeGrid: Cell[][],
  writeLastMoveGrid: MoveDirection[][],
  writeColorGrid: string[][],
  particles: Particle[],
  frameCount: number,
): {
  newParticles: Particle[];
} => {


  // Add a guard clause to check if the grid is initialized
  if (!readGrid || readGrid.length === 0 || !readGrid[0] || readGrid[0].length === 0) {
    // Grid not initialized yet, return current state
    return { newParticles: particles };
  }

  const height = readGrid.length;
  const width = readGrid[0].length;
  const elements = useGameStore.getState().elements;
  const particleInteractionRules = useGameStore.getState().particleInteractionRules;

  if (Object.keys(elements).length === 0) {
    // Elements not loaded yet, return current state
    return { newParticles: particles };
  }

  const spawnedParticles: Particle[] = [];

  // --- PASS 1: MOVEMENT ---
  const moved = Array(height).fill(null).map(() => Array(width).fill(false));
  const scanRight = frameCount % 2 === 0;

  for (let y = height - 1; y >= 0; y--) {
    for (let i = 0; i < width; i++) {
      const x = scanRight ? i : width - 1 - i;

      if (moved[y][x]) continue;

      const element = readGrid[y][x].type;
      if (element === 'EMPTY') {
        writeGrid[y][x] = { type: 'EMPTY' };
        writeColorGrid[y][x] = elements.EMPTY.color;
        continue;
      }

      const behavior = behaviors[element];
      if (behavior) {
        const newParticle = behavior({
          grid: readGrid, // Pass read-only grid
          lastMoveGrid: readLastMoveGrid,
          colorGrid: readColorGrid,
          newGrid: writeGrid, // Pass writable grid
          newLastMoveGrid: writeLastMoveGrid,
          newColorGrid: writeColorGrid,
          moved,
          x,
          y,
          width,
          height,
          scanRight,
        });
        if (newParticle) {
          spawnedParticles.push(newParticle);
        }
      } else {
        // If no behavior, the element stays in place
        writeGrid[y][x] = readGrid[y][x];
        writeColorGrid[y][x] = readColorGrid[y][x];
      }
    }
  }

  // --- PASS 2: TRANSFORMATIONS ---
  for (let y = height - 1; y >= 0; y--) {
    for (let i = 0; i < width; i++) {
      const x = scanRight ? i : width - 1 - i;
      const newParticle = handleTransformations({
        grid: writeGrid, // Read from and write to the same grid
        newGrid: writeGrid, // Pass it as newGrid as well
        newColorGrid: writeColorGrid,
        x, y, width, height,
      });
      if (newParticle) {
        spawnedParticles.push(newParticle);
      }
    }
  }

  // --- PASS 2.5: PLANT GROWTH ---
  handlePlantGrowth(writeGrid, writeGrid, writeColorGrid, width, height);

  // Color variation is now handled by the hasColorVariation property in elements.json

  // Update color grid after transformations
  // This loop is now more complex because we don't have a separate `gridAfterMove`
  // We can skip this color update for now, as the main color update will happen in the behavior itself.
  // This is a potential area for further optimization.

  // --- PASS 3: PARTICLE SIMULATION & DEEPENING ---
  let nextParticleId = useGameStore.getState().nextParticleId;
  for (const p of spawnedParticles) {
    if (p.id === -1) {
      p.id = nextParticleId++;
    }
  }

  const allParticles = particles.concat(spawnedParticles);

  // Handle Ether particles first
  const etherResult = handleEtherParticles({
    particles: allParticles,
    grid: writeGrid, // Use the latest grid state
    width,
    height,
    rules: particleInteractionRules,
  });

  const thunderResult = handleThunderParticles({
    particles: etherResult.updatedParticles,
    grid: etherResult.updatedGrid, // Use the grid potentially modified by Ether
    width,
    height,
    spawnedParticles, // Pass the array to be populated
    nextParticleId,   // Pass the current ID counter
  });

  // Then, handle Fire particles with the result from the Thunder simulation
  const fireResult = handleFireParticles({
    particles: thunderResult.updatedParticles,
    grid: thunderResult.updatedGrid, // Use the grid potentially modified by Thunder
    width,
    height,
    nextParticleId: thunderResult.nextParticleId, // Pass the updated ID counter
  });

  // Update the counter with the value returned from fire behavior and set it in the store
  nextParticleId = fireResult.nextParticleId;
  useGameStore.setState({ nextParticleId });

  // If any simulation changed the grid, we need to copy the changes back
  if (etherResult.gridChanged || thunderResult.gridChanged || fireResult.gridChanged) {
    const finalGrid = fireResult.gridChanged ? fireResult.updatedGrid : (thunderResult.gridChanged ? thunderResult.updatedGrid : etherResult.updatedGrid);
    for (let y = 0; y < height; y++) {
      for (let x = 0; x < width; x++) {
        if (writeGrid[y][x].type !== finalGrid[y][x].type) {
          writeGrid[y][x] = finalGrid[y][x];
          const newType = finalGrid[y][x].type;
          const baseColor = elements[newType]?.color || '#000000';
          writeColorGrid[y][x] = elements[newType]?.hasColorVariation ? varyColor(baseColor) : baseColor;
        }
      }
    }
  }

  return { newParticles: fireResult.updatedParticles };
};

// Calculate statistics for elements in the grid
export const calculateStats = (grid: Cell[][], particles: Particle[]): Record<ElementName | "ETHER" | "THUNDER", number> => {
  // Get the current stats object from the store to ensure all elements are present
  const stats = useGameStore.getState().stats;

  // Reset all counts to zero
  for (const key in stats) {
    stats[key as keyof typeof stats] = 0;
  }

  // Recalculate counts from the grid
  for (let y = 0; y < grid.length; y++) {
    for (let x = 0; x < grid[y].length; x++) {
      const element = grid[y][x].type;
      if (element in stats && element !== 'EMPTY') { // Do not count EMPTY
        stats[element]++;
      }
    }
  }

  // Recalculate counts from particles
  for (const particle of particles) {
    if (particle.type in stats) {
      stats[particle.type as keyof typeof stats]++;
    }
  }

  return stats;
};