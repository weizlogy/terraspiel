import { type Cell, type Particle } from "../../types/elements";
import useGameStore from "../../stores/gameStore";

interface BehaviorContext {
  grid: Cell[][];
  colorGrid: string[][];
  newGrid: Cell[][];
  newColorGrid: string[][];
  moved: boolean[][];
  x: number;
  y: number;
  width: number;
  height: number;
}

export const handleCloud = ({
  grid,
  colorGrid,
  newGrid,
  newColorGrid,
  moved,
  x,
  y,
  width,
  height,
}: BehaviorContext): Particle | null => {
  const { elements, colorVariations } = useGameStore.getState();
  if (Object.keys(elements).length === 0) return null;

  const cell = grid[y][x];
  const color = colorGrid[y][x];
  let hasMoved = false;
  let spawnedParticle: Particle | null = null;

  let { 
    rainCounter = 0, 
    chargeCounter = 0, 
    decayCounter = 0, 
    rainThreshold, 
    chargeThreshold 
  } = cell;

  // Initialize random thresholds if they don't exist
  if (rainThreshold === undefined) {
    rainThreshold = 100 + Math.floor(Math.random() * 40) - 20; // 80-120
  }
  if (chargeThreshold === undefined) {
    chargeThreshold = 800 + Math.floor(Math.random() * 400) - 200; // 600-1000
  }

  // --- Cloud Interaction Logic ---
  let isTouchingCloud = false;
  for (let i = -1; i <= 1; i++) {
    for (let j = -1; j <= 1; j++) {
      if (i === 0 && j === 0) continue;
      const nx = x + j;
      const ny = y + i;
      if (ny >= 0 && ny < height && nx >= 0 && nx < width && grid[ny][nx].type === 'CLOUD') {
        isTouchingCloud = true;
        break;
      }
    }
    if (isTouchingCloud) break;
  }

  if (isTouchingCloud) {
    rainCounter += 1; // Increase rain counter
    chargeCounter += 1; // Increase charge counter
  }
  // --- End Cloud Interaction Logic ---

  // --- Decay Logic ---
  const decayChance = 0.02;
  const decayThreshold = 100;

  if (Math.random() < decayChance) {
    decayCounter++;
  }

  if (decayCounter >= decayThreshold) {
    newGrid[y][x] = { type: 'EMPTY' };
    newColorGrid[y][x] = elements.EMPTY.color;
    moved[y][x] = true;
    return null; // Cloud disappears, no further action needed
  }
  // --- End Decay Logic ---

  // --- Rain Logic ---
  const rainChance = 0.02; // Lowered from 0.05

  if (Math.random() < rainChance) {
    rainCounter++;
  }

  if (rainCounter >= rainThreshold) {
    // Try to rain below
    if (y + 1 < height && grid[y + 1][x].type === 'EMPTY' && !moved[y + 1][x]) {
      newGrid[y + 1][x] = { type: 'WATER' };
      const variations = colorVariations.get('WATER');
      if (variations && variations.length > 0) {
        newColorGrid[y + 1][x] = variations[Math.floor(Math.random() * variations.length)];
      } else {
        newColorGrid[y + 1][x] = elements.WATER.color;
      }
      moved[y + 1][x] = true;
      rainCounter = 0; // Reset counter
      decayCounter += 10; // Increment decay counter on rain
    }
  }
  // --- End Rain Logic ---

  // --- Charge Logic ---
  const chargeChance = 0.05;

  if (Math.random() < chargeChance) {
    chargeCounter++;
  }

  if (chargeCounter >= chargeThreshold) {
    spawnedParticle = {
      id: -1, // Temporary ID, will be assigned in physics engine
      px: x + 0.5,
      py: y + 0.5,
      vx: Math.random() - 0.5,
      vy: Math.random() * 2 + 2,
      type: 'THUNDER',
      life: 60,
    };
    chargeCounter = 0; // Reset counter
  }
  // --- End Charge Logic ---


  // Movement priority: up > diagonally up > sideways, with some randomness
  const upwardChance = 0.7; // High chance to move up
  const spreadChance = 0.5; // Moderate chance to spread

  // Get the new cell state after potential changes
  const currentCell = newGrid[y][x].type !== 'EMPTY' ? newGrid[y][x] : grid[y][x];
  const updatedCounters = { rainCounter, chargeCounter, decayCounter, rainThreshold, chargeThreshold };


  // 1. Try moving up
  if (y > 0 && !hasMoved && Math.random() < upwardChance) {
    const targetCell = grid[y - 1][x];
    if ((targetCell.type === 'EMPTY' || targetCell.type === 'WATER') && !moved[y - 1][x]) {
      const isTargetWater = targetCell.type === 'WATER';

      // Move CLOUD up
      newGrid[y - 1][x] = { ...currentCell, type: 'CLOUD', ...updatedCounters };
      newColorGrid[y - 1][x] = color;
      moved[y - 1][x] = true;

      // Replace original position
      if (isTargetWater) {
        newGrid[y][x] = targetCell; // Move WATER down
        newColorGrid[y][x] = colorGrid[y - 1][x];
      } else {
        newGrid[y][x] = { type: 'EMPTY' };
        newColorGrid[y][x] = elements.EMPTY.color;
      }
      moved[y][x] = true;
      hasMoved = true;
    }
  }

  // 2. Try moving diagonally up
  if (y > 0 && !hasMoved && Math.random() < spreadChance) {
    const diagonalDirections = [-1, 1];
    if (Math.random() > 0.5) diagonalDirections.reverse();

    for (const dx of diagonalDirections) {
      const nx = x + dx;
      if (nx >= 0 && nx < width) {
        const targetCell = grid[y - 1][nx];
        if ((targetCell.type === 'EMPTY' || targetCell.type === 'WATER') && !moved[y - 1][nx]) {
          const isTargetWater = targetCell.type === 'WATER';

          // Move CLOUD up-diagonally
          newGrid[y - 1][nx] = { ...currentCell, type: 'CLOUD', ...updatedCounters };
          newColorGrid[y - 1][nx] = color;
          moved[y - 1][nx] = true;

          // Replace original position
          if (isTargetWater) {
            newGrid[y][x] = targetCell; // Move WATER to original spot
            newColorGrid[y][x] = colorGrid[y - 1][nx];
          } else {
            newGrid[y][x] = { type: 'EMPTY' };
            newColorGrid[y][x] = elements.EMPTY.color;
          }
          moved[y][x] = true;
          hasMoved = true;
          break;
        }
      }
    }
  }

  // 3. Try moving sideways
  if (!hasMoved && Math.random() < spreadChance) {
    const directions = [-1, 1];
    if (Math.random() > 0.5) directions.reverse();

    for (const dx of directions) {
      const nx = x + dx;
      if (nx >= 0 && nx < width) {
        const targetCell = grid[y][nx];
        if ((targetCell.type === 'EMPTY' || targetCell.type === 'WATER') && !moved[y][nx]) {
          const isTargetWater = targetCell.type === 'WATER';

          // Move CLOUD sideways
          newGrid[y][nx] = { ...currentCell, type: 'CLOUD', ...updatedCounters };
          newColorGrid[y][nx] = color;
          moved[y][nx] = true;

          // Replace original position
          if (isTargetWater) {
            newGrid[y][x] = targetCell; // Move WATER to original spot
            newColorGrid[y][x] = colorGrid[y][nx];
          } else {
            newGrid[y][x] = { type: 'EMPTY' };
            newColorGrid[y][x] = elements.EMPTY.color;
          }
          moved[y][x] = true;
          hasMoved = true;
          break;
        }
      }
    }
  }

  // If the cloud hasn't moved, update its counter in the new grid
  if (!hasMoved) {
    newGrid[y][x] = { ...currentCell, type: 'CLOUD', ...updatedCounters };
    newColorGrid[y][x] = color;
  }

  return spawnedParticle;
};