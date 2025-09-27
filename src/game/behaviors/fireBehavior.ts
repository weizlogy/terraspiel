import { type Cell, type ElementName } from "../../types/elements";
import useGameStore from "../../stores/gameStore";

interface BehaviorContext {
  grid: Cell[][];
  newGrid: Cell[][];
  newColorGrid: string[][];
  moved: boolean[][];
  x: number;
  y: number;
  width: number;
  height: number;
}

// 燃焼ルール定義
const COMBUSTION_RULES: Partial<Record<ElementName, { selfTo: ElementName, neighborTo: ElementName, threshold: number }>> = {
  'SOIL': { selfTo: 'SAND', neighborTo: 'FIRE', threshold: 30 },
  'CLAY': { selfTo: 'STONE', neighborTo: 'FIRE', threshold: 50 },
  'MUD':  { selfTo: 'SOIL', neighborTo: 'FIRE', threshold: 20 },
};

export const handleFire = ({
  grid,
  newGrid,
  newColorGrid,
  moved,
  x,
  y,
  width,
  height,
}: BehaviorContext): void => {
  const elements = useGameStore.getState().elements;
  if (Object.keys(elements).length === 0) return;

  const directions = [
    [-1, -1], [-1, 0], [-1, 1],
    [0, -1],          [0, 1],
    [1, -1], [1, 0], [1, 1]
  ];

  // Shuffle directions to check a random neighbor first
  for (let i = directions.length - 1; i > 0; i--) {
    const j = Math.floor(Math.random() * (i + 1));
    [directions[i], directions[j]] = [directions[j], directions[i]];
  }

  for (const [dy, dx] of directions) {
    const nx = x + dx;
    const ny = y + dy;

    if (nx >= 0 && nx < width && ny >= 0 && ny < height) {
      const neighborType = grid[ny][nx].type;

      // WATERに接触した場合は、燃焼処理を行わず終了
      if (neighborType === 'WATER') {
        return; // WATERに触れたので燃焼処理をスキップ
      }

      const rule = COMBUSTION_RULES[neighborType];

      // 燃焼ルールが存在する場合（カウンターはmovedチェック無しに毎フレーム増やす）
      if (rule) {
        const neighborCell = newGrid[ny][nx];
        const currentCounter = neighborCell.counter || 0;
        const newCounter = currentCounter + 1;
        neighborCell.counter = newCounter; // カウンターを更新

        // 閾値に達し、かつ隣接セルがまだ動いていない場合、変化を起こす (movedチェックあり)
        if (newCounter >= rule.threshold && !moved[ny][nx]) {
          // --- COMBUSTION ---
          // The FIRE's current position changes
          newGrid[y][x] = { type: rule.selfTo };
          newColorGrid[y][x] = elements[rule.selfTo].color;

          // The neighbor's position becomes FIRE
          newGrid[ny][nx] = { type: rule.neighborTo, counter: 0 }; // Reset counter
          newColorGrid[ny][nx] = elements[rule.neighborTo].color;

          moved[y][x] = true;
          moved[ny][nx] = true;

          return; // Exit after one reaction (変化が起こったので終了)
        }
        // カウンターが増えただけの場合は、他の方向もチェックするためreturnしない
      }
    }
  }
};