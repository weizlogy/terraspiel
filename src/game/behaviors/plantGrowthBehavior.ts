import { type Cell } from "../../types/elements";
import useGameStore from "../../stores/gameStore";
import { varyColor } from "../../utils/colors";

const DECAY_THRESHOLD = 500;
const GROWTH_THRESHOLD = 100;
const STEM_GROW_CHANCE = 0.1;
const LEAF_GROW_CHANCE = 0.2;
const FLOWER_GROW_CHANCE = 0.05;
const GROUND_COVER_SPREAD_CHANCE = 0.3;

export const handlePlantGrowth = (
  grid: Cell[][],
  newGrid: Cell[][],
  newColorGrid: string[][],
  width: number,
  height: number
) => {
  const elements = useGameStore.getState().elements;
  const plantElement = elements.PLANT;
  if (!plantElement || !plantElement.partColors) return; // Guard against missing data

  const leafColor = plantElement.partColors.leaf;
  const flowerColor = plantElement.partColors.flower;

  for (let y = 0; y < height; y++) {
    for (let x = 0; x < width; x++) {
      const cell = grid[y][x];
      const newCell = newGrid[y][x];

      if (cell.type !== 'PLANT') {
        continue;
      }

      // --- 1. Decay Logic ---
      // Only body parts (stem, ground_cover) decay and grow
      if (newCell.plantMode === 'stem' || newCell.plantMode === 'ground_cover') {
        const decayCounter = (newCell.decayCounter || 0) + 1;
        if (decayCounter > DECAY_THRESHOLD * (0.8 + Math.random() * 0.4)) {
          newGrid[y][x] = { type: 'WITHERED_PLANT' };
          newColorGrid[y][x] = varyColor(elements.WITHERED_PLANT.color);
          continue;
        }
        newGrid[y][x].decayCounter = decayCounter;

        // --- 2. Growth Logic ---
        const growthCounter = (newCell.counter || 0) + 1;
        if (growthCounter < GROWTH_THRESHOLD) {
          newGrid[y][x].counter = growthCounter;
          continue;
        }
        
        newGrid[y][x].counter = 0; // Reset growth counter

        if (newCell.plantMode === 'stem') {
          // a. Grow upwards
          const upY = y - 1;
          if (upY >= 0 && grid[upY][x].type === 'EMPTY' && Math.random() < STEM_GROW_CHANCE) {
            newGrid[upY][x] = { type: 'PLANT', plantMode: 'stem', counter: 0, decayCounter: 0 };
            newColorGrid[upY][x] = varyColor(plantElement.color);
          }

          // b. Grow leaves and flowers on sides
          for (const dir of [-1, 1]) {
            const sideX = x + dir;
            if (sideX >= 0 && sideX < width && grid[y][sideX].type === 'EMPTY') {
              if (Math.random() < LEAF_GROW_CHANCE) {
                newGrid[y][sideX] = { type: 'PLANT', plantMode: 'leaf' };
                newColorGrid[y][sideX] = varyColor(leafColor);
              } else if (Math.random() < FLOWER_GROW_CHANCE) {
                newGrid[y][sideX] = { type: 'PLANT', plantMode: 'flower' };
                newColorGrid[y][sideX] = varyColor(flowerColor);
              }
            }
          }
        } else if (newCell.plantMode === 'ground_cover') {
          const dir = Math.random() < 0.5 ? -1 : 1;
          const nx = x + dir;
          if (nx >= 0 && nx < width && grid[nx][y].type === 'EMPTY' && Math.random() < GROUND_COVER_SPREAD_CHANCE) {
              const belowY = y + 1;
              if (belowY < height && grid[belowY][nx].type !== 'EMPTY' && elements[grid[belowY][nx].type]?.isStatic === false) {
                   newGrid[nx][y] = { type: 'PLANT', plantMode: 'ground_cover', counter: 0, decayCounter: 0 };
                   newColorGrid[nx][y] = varyColor(plantElement.color);
              }
          }
        }
      }
    }
  }
};