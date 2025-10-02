import { type Cell, type ElementName } from "../../types/elements";
import useGameStore from "../../stores/gameStore";
import { varyColor } from "../../utils/colors";

interface BehaviorContext {
  grid: Cell[][];
  newGrid: Cell[][];
  newColorGrid: string[][];
  colorGrid: string[][];
  moved: boolean[][];
  x: number;
  y: number;
  width: number;
  height: number;
}

const FIRE_LIFESPAN = 100; // Fire's lifespan in frames

// 燃焼ルール定義
const COMBUSTION_RULES: Partial<Record<ElementName, { selfTo: ElementName, neighborTo: ElementName, threshold: number }>> = {
  'SOIL': { selfTo: 'SAND', neighborTo: 'FIRE', threshold: 30 },
  'CLAY': { selfTo: 'STONE', neighborTo: 'FIRE', threshold: 50 },
  'MUD':  { selfTo: 'SOIL', neighborTo: 'FIRE', threshold: 20 },
  'OIL': { selfTo: 'FIRE', neighborTo: 'FIRE', threshold: 1 },
  'PLANT': { selfTo: 'FIRE', neighborTo: 'FIRE', threshold: 5 },
};

// Color variation is now handled by the hasColorVariation property in elements.json

export const handleFire = ({
  grid,
  newGrid,
  newColorGrid,
  colorGrid,
  moved,
  x,
  y,
  width,
  height,
}: BehaviorContext): void => {
  const elements = useGameStore.getState().elements;
  if (Object.keys(elements).length === 0) return;

  let hasBurnedSomething = false;

  // 隣接セルをチェックする前に、WATER接触をチェックする (ルール6)
  const directions = [
    [-1, -1], [-1, 0], [-1, 1],
    [0, -1],          [0, 1],
    [1, -1], [1, 0], [1, 1]
  ];

  // Shuffle directions to check a random neighbor first (なくても良いかもしれない)
  for (let i = directions.length - 1; i > 0; i--) {
    const j = Math.floor(Math.random() * (i + 1));
    [directions[i], directions[j]] = [directions[j], directions[i]];
  }

  // 隣接セルをチェック
  for (const [dy, dx] of directions) {
    const nx = x + dx;
    const ny = y + dy;

    if (nx >= 0 && nx < width && ny >= 0 && ny < height) {
      const neighborType = grid[ny][nx].type;

      // WATERに接触した場合は、消火
      if (neighborType === 'WATER') {
        newGrid[y][x] = { type: 'EMPTY' };
        newColorGrid[y][x] = elements.EMPTY.color;
        moved[y][x] = true;
        return;
      }

      const rule = COMBUSTION_RULES[neighborType];

      // 燃焼ルールが存在する場合
      if (rule) {
        const neighborCell = newGrid[ny][nx];
        // burningProgressを更新
        const currentProgress = neighborCell.burningProgress || 0;
        const newProgress = currentProgress + 1;
        neighborCell.burningProgress = newProgress;

        // 閾値に達したら変化
        if (newProgress >= rule.threshold) {
          // --- COMBUSTION ---
          // The FIRE's current position changes
          newGrid[y][x] = { type: rule.selfTo, life: FIRE_LIFESPAN };
          newColorGrid[y][x] = elements[rule.selfTo]?.hasColorVariation ? varyColor(elements[rule.selfTo].color) : elements[rule.selfTo].color;

          // The neighbor's position becomes FIRE
          newGrid[ny][nx] = { type: rule.neighborTo, burningProgress: 0, life: FIRE_LIFESPAN }; // Reset burningProgress
          newColorGrid[ny][nx] = elements[rule.neighborTo]?.hasColorVariation ? varyColor(elements[rule.neighborTo].color) : elements[rule.neighborTo].color;

          // movedフラグを設定して、そのフレームで他の処理が入らないようにする
          moved[y][x] = true;
          moved[ny][nx] = true;
          
          hasBurnedSomething = true;

          // 1つの変化が起こったら終了 (他の方向はチェックしない)
          return;
        }
      }
    }
  }

  // 燃やすものがなかった場合、lifeを減らす
  if (!hasBurnedSomething) {
    const currentLife = grid[y][x].life ?? FIRE_LIFESPAN;
    const newLife = currentLife - 1;

    if (newLife <= 0) {
      newGrid[y][x] = { type: 'EMPTY' };
      newColorGrid[y][x] = elements.EMPTY.color;
    } else {
      newGrid[y][x] = { ...grid[y][x], life: newLife };
      newColorGrid[y][x] = colorGrid[y][x]; // Keep the same color
    }
    moved[y][x] = true;
  }
};