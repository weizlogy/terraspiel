import { type Cell, type RuleCondition, type SurroundingCondition, type EnvironmentCondition, type Particle, type Element, type ElementName, type SurroundingAttributeCondition } from "../types/elements";
import useGameStore from "../stores/gameStore";
import { varyColor } from "../utils/colors";

interface BehaviorContext {
  grid: Cell[][];
  newGrid: Cell[][];
  newColorGrid: string[][];
  x: number;
  y: number;
  width: number;
  height: number;
}

// Helper to check if a single condition is met
const checkCondition = (condition: RuleCondition, grid: Cell[][], x: number, y: number, width: number, height: number, elements: Record<ElementName, Element>): boolean => {
  switch (condition.type) {
    case 'surrounding': {
      const { element, min = 0, max = 8 } = condition as SurroundingCondition;
      let count = 0;
      for (let j = -1; j <= 1; j++) {
        for (let i = -1; i <= 1; i++) {
          if (i === 0 && j === 0) continue;
          const nx = x + i;
          const ny = y + j;
          if (nx >= 0 && nx < width && ny >= 0 && ny < height && grid[ny][nx].type === element) {
            count++;
          }
        }
      }
      return count >= min && count <= max;
    }
    case 'environment': {
      const { element, presence, radius } = condition as EnvironmentCondition;
      let elementFound = false;
      for (let j = -radius; j <= radius; j++) {
        for (let i = -radius; i <= radius; i++) {
          if (i === 0 && j === 0) continue;
          const nx = x + i;
          const ny = y + j;
          if (nx >= 0 && nx < width && ny >= 0 && ny < height && grid[ny][nx].type === element) {
            elementFound = true;
            break;
          }
        }
        if (elementFound) break;
      }

      return presence === 'exists' ? elementFound : !elementFound;
    }
    case 'surroundingAttribute': {
      const { attribute, value, min = 0, max = 8 } = condition as SurroundingAttributeCondition;
      let count = 0;
      for (let j = -1; j <= 1; j++) {
        for (let i = -1; i <= 1; i++) {
          if (i === 0 && j === 0) continue;
          const nx = x + i;
          const ny = y + j;
          if (nx >= 0 && nx < width && ny >= 0 && ny < height) {
            const neighborType = grid[ny][nx].type;
            const elementDef = elements[neighborType];
            if (elementDef && elementDef[attribute] === value) {
              count++;
            }
          }
        }
      }
      return count >= min && count <= max;
    }
    default:
      return true;
  }
};

export const handleTransformations = ({
  grid,
  newGrid,
  newColorGrid,
  x,
  y,
  width,
  height,
}: BehaviorContext): Particle | null => {
  const { transformationRules, nextParticleId, elements } = useGameStore.getState();
  const cell = grid[y][x];
  const applicableRules = transformationRules.filter(rule => rule.from === cell.type);

  if (applicableRules.length === 0) {
    return null;
  }

  // Find all rules that meet their conditions
  const metRules = applicableRules.filter(rule => 
    rule.conditions.every(cond => checkCondition(cond, grid, x, y, width, height, elements))
  );

  if (metRules.length > 0) {
    // Prioritize the first rule that met its conditions
    const rule = metRules[0];

    if (Math.random() < rule.probability) {
      const currentCounter = newGrid[y][x].counter || 0;
      const newCounter = currentCounter + 1;

      if (newCounter >= rule.threshold) {
        // --- Transformation occurs ---
        if (rule.consumes) {
          let consumed = false;
          const directions = [-1, 0, 1].sort(() => Math.random() - 0.5);
          for (const j of directions) {
            for (const i of directions) {
              if (i === 0 && j === 0) continue;
              const nx = x + i;
              const ny = y + j;
              if (nx >= 0 && nx < width && ny >= 0 && ny < height && grid[ny][nx].type === rule.consumes) {
                newGrid[ny][nx] = { type: 'EMPTY' };
                // Also update color of consumed cell
                newColorGrid[ny][nx] = elements.EMPTY.color;
                consumed = true;
                break;
              }
            }
            if (consumed) break;
          }
        }

        const fromType = grid[y][x].type;
        newGrid[y][x] = { type: rule.to, counter: 0 };
        
        // Also update the color
        const newElement = elements[rule.to];
        if (newElement) {
          newColorGrid[y][x] = varyColor(newElement.color);
        }

        const ETHER_SPAWN_CHANCE = 0.005;
        if (fromType !== rule.to && Math.random() < ETHER_SPAWN_CHANCE) {
          const vx = (Math.random() - 0.5) * 0.3;
          const vy = (Math.random() - 0.5) * 0.3;
          
          const newParticle: Particle = {
            id: nextParticleId,
            px: x + 0.5,
            py: y + 0.5,
            vx,
            vy,
            type: 'ETHER',
            life: 150,
          };
          useGameStore.setState({ nextParticleId: nextParticleId + 1 });
          return newParticle;
        }
      } else {
        newGrid[y][x].counter = newCounter;
      }
    }
  } else {
    // No rules met conditions, reset counter if it was running
    if (newGrid[y][x].counter && newGrid[y][x].counter > 0) {
      newGrid[y][x].counter = 0;
    }
  }

  return null;
};
