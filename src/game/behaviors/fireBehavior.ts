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

      // WATERに接触した場合は、燃焼処理を行わず終了 (WATERは燃焼対象でもルールにない)
      if (neighborType === 'WATER') {
        return; // WATERに触れたので燃焼処理をスキップ
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
          newGrid[y][x] = { type: rule.selfTo };
          newColorGrid[y][x] = elements[rule.selfTo].color;

          // The neighbor's position becomes FIRE
          newGrid[ny][nx] = { type: rule.neighborTo, burningProgress: 0 }; // Reset burningProgress
          newColorGrid[ny][nx] = elements[rule.neighborTo].color;

          // movedフラグを設定して、そのフレームで他の処理が入らないようにする
          moved[y][x] = true;
          moved[ny][nx] = true;

          // 1つの変化が起こったら終了 (他の方向はチェックしない)
          return; // Exit after one reaction (変化が起こったので終了)
        }
        // 閾値に達していなければ、他の方向もチェックする
        // (以前はカウンター更新後もreturnしていたが、変化が起こらなければ続ける)
      }
      // ルールがなければ、他の方向をチェックする
    }
  }
  // 全方向チェックしても変化がなければ終了
};