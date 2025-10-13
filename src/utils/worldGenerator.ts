import { makeNoise2D } from 'fast-simplex-noise';
import type { Cell, ElementName } from '../types/elements';

interface WorldGenerationParams {
  width: number;
  height: number;
  seed?: number;
}

interface CellularAutomataParams {
  birthLimit: number;
  deathLimit: number;
  steps: number;
}

/**
 * Generate terrain using Simplex + Cellular Automata
 */
export const generateTerrain = (params: WorldGenerationParams): Cell[][] => {
  const { width, height, seed = Math.random() } = params;
  
  const simplexNoise = makeNoise2D(() => seed);

  let grid: Cell[][] = Array(height)
    .fill(0)
    .map(() => Array(width).fill(0).map(() => ({ type: 'EMPTY' })));

  grid = generateSurface(grid, { width, height }, simplexNoise, seed);
  grid = generateUnderground(grid, { width, height }, simplexNoise, seed);

  return grid;
};

/**
 * Generate surface terrain: Simplex + Cellular Automata
 */
const generateSurface = (
  grid: Cell[][], 
  params: { width: number; height: number }, 
  simplexNoise: (x: number, y: number) => number,
  seed: number
): Cell[][] => {
  const { width, height } = params;

  // Cloud generation
  const cloudNoiseFrequency = 80;
  for (let y = 0; y < height * 0.3; y++) { // Clouds in the top 30% of the world
    for (let x = 0; x < width; x++) {
      const noiseValue = (simplexNoise(x / cloudNoiseFrequency, y / cloudNoiseFrequency) + 1) / 2;
      if (noiseValue > 0.85) { // Significantly reduced cloud generation
        grid[y][x] = { type: 'CLOUD' };
      }
    }
  }

  const elevationFrequency = 100;
  const strataNoiseFrequency = 50;
  const surfaceElements: ElementName[] = ['SOIL', 'SAND', 'CLAY', 'PEAT', 'FERTILE_SOIL', 'STONE', 'BASALT', 'CRYSTAL', 'RUBY', 'SAPPHIRE', 'EMERALD', 'AMETHYST', 'GARNET', 'ELECTRUM', 'MAGMA'];

  for (let x = 0; x < width; x++) {
    const surfaceY = height * 0.4 + ((simplexNoise(x / elevationFrequency, seed) + 1) / 2) * (height * 0.3);

    for (let y = Math.floor(surfaceY); y < height; y++) {
      if (grid[y][x].type === 'EMPTY') { // Don't overwrite clouds
        const strataNoise = (simplexNoise(x / strataNoiseFrequency, y / strataNoiseFrequency) + 1) / 2;
        const randomIndex = Math.floor(strataNoise * surfaceElements.length);
        let cellType = surfaceElements[randomIndex];

        grid[y][x] = { type: cellType };
      }
    }
  }

  // Smoothing and contextual passes
  const caParams: CellularAutomataParams = { birthLimit: 4, deathLimit: 4, steps: 2 };
  for (let step = 0; step < caParams.steps; step++) {
    grid = applyCellularAutomata(grid, caParams);
  }

  // Plant and Seed generation
  for (let x = 0; x < width; x++) {
    for (let y = 1; y < height; y++) {
      const surfaceBlock = grid[y][x].type;
      const blockAbove = grid[y-1][x].type;
      if ((surfaceBlock === 'SOIL' || surfaceBlock === 'FERTILE_SOIL') && blockAbove === 'EMPTY') {
        if (Math.random() < 0.15) grid[y-1][x] = { type: 'PLANT' };
        else if (Math.random() < 0.05) grid[y-1][x] = { type: 'SEED' };
        break;
      }
    }
  }
  
  return grid;
};

/**
 * Generate underground terrain: Simplex + CA + Ores
 */
const generateUnderground = (
  grid: Cell[][],
  params: { width: number; height: number },
  simplexNoise: (x: number, y: number) => number
): Cell[][] => {
  const { width, height } = params;
  const caveNoiseFrequency = 40;
  const strataNoiseFrequency = 60;
  const oilNoiseFrequency = 100;
  const bedrockLevel = height * 0.9;

  // Randomize the density of the world
  const minThreshold = 0.7; // Corresponds to ~2x the previous solid amount
  const maxThreshold = 0.875; // Corresponds to ~2.5x the previous solid amount
  const randomCaveThreshold = minThreshold + Math.random() * (maxThreshold - minThreshold);

  const ores: { type: ElementName; frequency: number; threshold: number; seed: number }[] = [
    { type: 'CRYSTAL', frequency: 30, threshold: 0.95 },
    { type: 'RUBY', frequency: 40, threshold: 0.97 },
    { type: 'ELECTRUM', frequency: 50, threshold: 0.98 },
    { type: 'SAPPHIRE', frequency: 45, threshold: 0.975 },
    { type: 'EMERALD', frequency: 48, threshold: 0.98 },
    { type: 'AMETHYST', frequency: 42, threshold: 0.97 },
    { type: 'GARNET', frequency: 46, threshold: 0.975 },
  ];

  let caveGrid: boolean[][] = Array(height).fill(false).map(() => Array(width).fill(false));
  let surfaceLevels = Array(width).fill(0);

  for (let x = 0; x < width; x++) {
    for (let y = 0; y < height; y++) {
      if (grid[y][x].type !== 'EMPTY' && grid[y][x].type !== 'CLOUD') {
        surfaceLevels[x] = y;
        break;
      }
    }
  }

  for (let y = 0; y < height; y++) {
    for (let x = 0; x < width; x++) {
      if (y > surfaceLevels[x] + 5) {
        const noiseValue = (simplexNoise(x / caveNoiseFrequency, y / caveNoiseFrequency) + 1) / 2;
        if (noiseValue > randomCaveThreshold) {
          caveGrid[y][x] = true;
        }
      }
    }
  }
  
  const caveCAParams: CellularAutomataParams = {
    birthLimit: 4,
    deathLimit: 3,
    steps: 5
  };

  for (let i = 0; i < caveCAParams.steps; i++) {
    caveGrid = applyCaveCA(caveGrid, caveCAParams);
  }

  for (let y = 0; y < height; y++) {
    for (let x = 0; x < width; x++) {
      if (grid[y][x].type === 'EMPTY' && y > surfaceLevels[x]) {
        if (y > bedrockLevel) {
          grid[y][x] = { type: 'BASALT' };
        } else {
          const strataNoise = (simplexNoise(x / strataNoiseFrequency, y / strataNoiseFrequency) + 1) / 2;
          if (strataNoise < 0.5) {
            grid[y][x] = { type: 'SOIL' };
          } else {
            grid[y][x] = { type: 'STONE' };
          }
        }
      }

      // Oil generation
      if (grid[y][x].type === 'CLAY' || grid[y][x].type === 'STONE') {
          const oilNoise = (simplexNoise(x / oilNoiseFrequency, y / oilNoiseFrequency) + 1) / 2;
          if (oilNoise > 0.8) { // Create large oil pockets
              grid[y][x] = { type: 'OIL' };
          }
      }

      const currentBlock = grid[y][x].type;
      if (currentBlock === 'STONE' || currentBlock === 'BASALT') {
        for (const ore of ores) {
          const oreNoiseValue = (simplexNoise(x / ore.frequency, y / ore.frequency) + 1) / 2;
          if (oreNoiseValue > ore.threshold) {
            grid[y][x] = { type: ore.type };
            break;
          }
        }
      }
      
      if (caveGrid[y][x]) {
        grid[y][x] = { type: 'EMPTY' };
      }
    }
  }

  return grid;
};

/**
 * Specialized CA for cave generation on a boolean grid.
 */
const applyCaveCA = (caveGrid: boolean[][], params: CellularAutomataParams): boolean[][] => {
  const { width, height } = { width: caveGrid[0].length, height: caveGrid.length };
  const newCaveGrid = caveGrid.map(row => [...row]);

  for (let y = 0; y < height; y++) {
    for (let x = 0; x < width; x++) {
      const neighborCount = countCaveNeighbors(caveGrid, x, y);
      
      if (caveGrid[y][x]) { // If cell is a cave
        if (neighborCount < params.deathLimit) {
          newCaveGrid[y][x] = false; // Fill in small caves
        }
      } else { // If cell is solid
        if (neighborCount > params.birthLimit) {
          newCaveGrid[y][x] = true; // Carve out new cave space
        }
      }
    }
  }
  return newCaveGrid;
};

/**
 * Helper to count neighbors for the boolean cave grid.
 */
const countCaveNeighbors = (caveGrid: boolean[][], x: number, y: number): number => {
  let count = 0;
  for (let i = -1; i <= 1; i++) {
    for (let j = -1; j <= 1; j++) {
      if (i === 0 && j === 0) continue;
      const nx = x + i;
      const ny = y + j;
      if (nx >= 0 && nx < caveGrid[0].length && ny >= 0 && ny < caveGrid.length) {
        if (caveGrid[ny][nx]) {
          count++;
        }
      } else {
        count++; // Treat edges as solid
      }
    }
  }
  return count;
};


/**
 * Apply Cellular Automata rules to the grid
 */
const applyCellularAutomata = (grid: Cell[][], params: CellularAutomataParams): Cell[][] => {
  const { width, height } = { width: grid[0].length, height: grid.length };
  const newGrid = grid.map(row => row.map(cell => ({ ...cell })));

  for (let y = 0; y < height; y++) {
    for (let x = 0; x < width; x++) {
      const neighborCount = countAliveNeighbors(grid, x, y);
      const cell = grid[y][x];

      if (cell.type !== 'EMPTY' && cell.type !== 'CLOUD') { // Clouds don't participate in CA
        if (neighborCount < params.deathLimit) {
          newGrid[y][x].type = 'EMPTY';
        }
      } else if (cell.type === 'EMPTY') { // If cell is "dead"
        if (neighborCount > params.birthLimit) {
          newGrid[y][x].type = 'STONE'; // Default birth material
        }
      }
    }
  }
  return newGrid;
};

/**
 * Helper function to count "alive" neighbors for Cellular Automata
 */
const countAliveNeighbors = (grid: Cell[][], x: number, y: number): number => {
  let count = 0;
  for (let i = -1; i <= 1; i++) {
    for (let j = -1; j <= 1; j++) {
      if (i === 0 && j === 0) continue;

      const nx = x + i;
      const ny = y + j;

      if (nx >= 0 && nx < grid[0].length && ny >= 0 && ny < grid.length) {
        if (grid[ny][nx].type !== 'EMPTY' && grid[ny][nx].type !== 'CLOUD') {
          count++;
        }
      } else {
        // Consider out-of-bounds as "alive" to create solid edges
        count++;
      }
    }
  }
  return count;
};
