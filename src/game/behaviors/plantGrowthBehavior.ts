import { type Cell } from "../../types/elements";
import useGameStore from "../../stores/gameStore";

const DECAY_THRESHOLD = 2000;
const GROWTH_THRESHOLD = 100;
const OIL_THRESHOLD = 1500;
const STEM_GROW_CHANCE = 0.1;
const LEAF_GROW_CHANCE = 0.2;
const FLOWER_GROW_CHANCE = 0.05;
const GROUND_COVER_SPREAD_CHANCE = 0.3;

const getRandomVariation = (variations: string[] | undefined, fallbackColor: string): string => {
  if (variations && variations.length > 0) {
    return variations[Math.floor(Math.random() * variations.length)];
  }
  return fallbackColor;
};

export const handlePlantGrowth = (
  grid: Cell[][],
  newGrid: Cell[][],
  newColorGrid: string[][],
  width: number,
  height: number
) => {
  const { elements, colorVariations } = useGameStore.getState();
  const plantElement = elements.PLANT;
  if (!plantElement || !plantElement.partColors) return;

  for (let y = 0; y < height; y++) {
    for (let x = 0; x < width; x++) {
      const cell = grid[y][x];
      const newCell = newGrid[y][x];

      if (cell.type !== 'PLANT') {
        continue;
      }

      // --- Handle Withered Plants: Transformation to OIL ---
      if (cell.plantMode === 'withered') {
        const oilCounter = (newCell.oilCounter || 0) + 1;
        if (oilCounter > OIL_THRESHOLD * (0.8 + Math.random() * 0.4)) {
          newGrid[y][x] = { type: 'OIL' };
          newColorGrid[y][x] = getRandomVariation(colorVariations.get('OIL'), elements.OIL.color);
        } else {
          // Ensure the cell remains a withered plant until it turns to oil
          newGrid[y][x] = { ...newCell, type: 'PLANT', plantMode: 'withered', oilCounter };
        }
        continue; // Withered plants don't grow or decay further
      }

      // --- Handle Living Plants (stem, ground_cover) ---
      if (newCell.plantMode === 'stem' || newCell.plantMode === 'ground_cover') {
        // 1. Decay Logic
        const decayCounter = (newCell.decayCounter || 0) + 1;
        if (decayCounter > DECAY_THRESHOLD * (0.8 + Math.random() * 0.4)) {
          newGrid[y][x] = { type: 'PLANT', plantMode: 'withered', oilCounter: 0 };
          newColorGrid[y][x] = getRandomVariation(colorVariations.get('PLANT_withered'), plantElement.partColors.withered);
          continue;
        }
        newGrid[y][x].decayCounter = decayCounter;

        // 2. Growth Logic
        const growthCounter = (newCell.counter || 0) + 1;
        if (growthCounter < GROWTH_THRESHOLD) {
          newGrid[y][x].counter = growthCounter;
          continue;
        }
        
        newGrid[y][x].counter = 0; // Reset growth counter

        if (newCell.plantMode === 'stem') {
          // a. Grow upwards
          const upY = y - 1;
          if (upY >= 0 && Math.random() < STEM_GROW_CHANCE) {
            const targetCell = grid[upY][x];
            if (targetCell.type !== 'PLANT') {
              newGrid[upY][x] = { type: 'PLANT', plantMode: 'stem', counter: 0, decayCounter: 0 };
              newColorGrid[upY][x] = getRandomVariation(colorVariations.get('PLANT'), plantElement.color);
            }
          }

          // b. Grow leaves and flowers on sides
          for (const dir of [-1, 1]) {
            const sideX = x + dir;
            if (sideX >= 0 && sideX < width && grid[y][sideX].type === 'EMPTY') {
              if (Math.random() < LEAF_GROW_CHANCE) {
                newGrid[y][sideX] = { type: 'PLANT', plantMode: 'leaf' };
                newColorGrid[y][sideX] = getRandomVariation(colorVariations.get('PLANT_leaf'), plantElement.partColors.leaf);
              } else if (Math.random() < FLOWER_GROW_CHANCE) {
                newGrid[y][sideX] = { type: 'PLANT', plantMode: 'flower' };
                newColorGrid[y][sideX] = getRandomVariation(colorVariations.get('PLANT_flower'), plantElement.partColors.flower);
              }
            }
          }
        } else if (newCell.plantMode === 'ground_cover') {
          const dir = Math.random() < 0.5 ? -1 : 1;
          const nx = x + dir;
          if (nx >= 0 && nx < width && grid[y][nx].type === 'EMPTY' && Math.random() < GROUND_COVER_SPREAD_CHANCE) {
              const belowY = y + 1;
              if (belowY < height && grid[belowY][nx].type !== 'EMPTY' && elements[grid[belowY][nx].type]?.isStatic === false) {
                   newGrid[y][nx] = { type: 'PLANT', plantMode: 'ground_cover', counter: 0, decayCounter: 0 };
                   newColorGrid[y][nx] = getRandomVariation(colorVariations.get('PLANT'), plantElement.color);
              }
          }
        }
      }
    }
  }
};