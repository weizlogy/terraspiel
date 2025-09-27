import { create } from 'zustand';
import { ELEMENTS, type ElementName, type Particle, type MoveDirection, type Cell, type ParticleType, type TransformationRule } from '../types/elements';
import { varyColor } from '../utils/colors';

interface GameState {
  selectedElement: ElementName;
  isPlaying: boolean;
  grid: Cell[][];
  lastMoveGrid: MoveDirection[][];
  colorGrid: string[][];
  particles: Particle[];
  nextParticleId: number;
  width: number;
  height: number;
  stats: Record<ElementName | 'ETHER', number>;
  fps: number;
  transformationRules: TransformationRule[];
  setSelectedElement: (element: ElementName) => void;
  setGrid: (grid: Cell[][]) => void;
  setLastMoveGrid: (lastMoveGrid: MoveDirection[][]) => void;
  setColorGrid: (colorGrid: string[][]) => void;
  setParticles: (particles: Particle[]) => void;
  addParticle: (x: number, y: number, type: ParticleType, vx?: number, vy?: number) => void;
  initializeGrid: () => void;
  clearGrid: () => void;
  randomizeGrid: () => void;
  updateStats: (stats: Record<ElementName | 'ETHER', number>) => void;
  setFps: (fps: number) => void;
  loadTransformationRules: () => Promise<void>;
}

const FIXED_WIDTH = 320;
const FIXED_HEIGHT = 180;

const useGameStore = create<GameState>()((set, get) => ({
  selectedElement: 'SOIL',
  isPlaying: true,
  grid: [],
  lastMoveGrid: [],
  colorGrid: [],
  particles: [],
  nextParticleId: 0,
  width: FIXED_WIDTH,
  height: FIXED_HEIGHT,
  stats: {
    EMPTY: 0, // Will be set to 0 to not count EMPTY elements
    SOIL: 0,
    WATER: 0,
    MUD: 0,
    FERTILE_SOIL: 0,
    PEAT: 0,
    CLOUD: 0,
    CLAY: 0,
    ETHER: 0,
  },
  fps: 0,
  transformationRules: [],
  setSelectedElement: (element) => set({ selectedElement: element }),
  setGrid: (grid) => set({ grid }),
  setLastMoveGrid: (lastMoveGrid) => set({ lastMoveGrid }),
  setColorGrid: (colorGrid) => set({ colorGrid }),
  setParticles: (particles) => set({ particles }),
  addParticle: (x, y, type, vx = 0, vy = 0) => {
    const newParticle: Particle = {
      id: get().nextParticleId,
      px: x,
      py: y,
      vx,
      vy,
      type,
      life: 150, // Default lifespan for ETHER
    };
    set((state) => ({
      particles: [...state.particles, newParticle],
      nextParticleId: state.nextParticleId + 1,
    }));
  },
  initializeGrid: () => {
    const width = FIXED_WIDTH;
    const height = FIXED_HEIGHT;
    const grid: Cell[][] = Array(height)
      .fill(null)
      .map(() => Array(width).fill(null).map(() => ({ type: 'EMPTY' })));
    const lastMoveGrid: MoveDirection[][] = Array(height)
      .fill(null)
      .map(() => Array(width).fill('NONE'));
    const colorGrid: string[][] = Array(height)
      .fill(null)
      .map(() => Array(width).fill(ELEMENTS.EMPTY.color));
    
    // Initialize stats (don't count EMPTY)
    const stats: Record<ElementName | 'ETHER', number> = {
      EMPTY: 0,
      SOIL: 0,
      WATER: 0,
      MUD: 0,
      FERTILE_SOIL: 0,
      PEAT: 0,
      CLOUD: 0,
      CLAY: 0,
      ETHER: 0,
    };
    
    set({ grid, lastMoveGrid, colorGrid, width, height, stats, particles: [], nextParticleId: 0 });
  },
  clearGrid: () => set((state) => {
    const grid: Cell[][] = Array(state.height)
      .fill(null)
      .map(() => Array(state.width).fill(null).map(() => ({ type: 'EMPTY' })));
    const lastMoveGrid: MoveDirection[][] = Array(state.height)
      .fill(null)
      .map(() => Array(state.width).fill('NONE'));
    const colorGrid: string[][] = Array(state.height)
      .fill(null)
      .map(() => Array(state.width).fill(ELEMENTS.EMPTY.color));
      
    const stats: Record<ElementName | 'ETHER', number> = {
      EMPTY: 0, // Don't count EMPTY
      SOIL: 0,
      WATER: 0,
      MUD: 0,
      FERTILE_SOIL: 0,
      PEAT: 0,
      CLOUD: 0,
      CLAY: 0,
      ETHER: 0,
    };
    
    return { grid, lastMoveGrid, colorGrid, stats, particles: [], nextParticleId: 0 };
  }),
  randomizeGrid: () => set((state) => {
    const gridElements: ElementName[] = ['SOIL', 'WATER']; // Elements that go into the grid
    const particleElementsForRandom: ParticleType[] = []; // Elements that become particles

    const newGrid: Cell[][] = Array(state.height)
      .fill(null)
      .map(() => Array(state.width).fill(null).map(() => ({ type: 'EMPTY' })));
    const newLastMoveGrid: MoveDirection[][] = Array(state.height)
      .fill(null)
      .map(() => Array(state.width).fill('NONE'));
    const newColorGrid: string[][] = Array(state.height)
      .fill(null)
      .map(() => Array(state.width).fill(ELEMENTS.EMPTY.color));
    
    const stats: Record<ElementName | 'ETHER', number> = {
      EMPTY: 0,
      SOIL: 0,
      WATER: 0,
      MUD: 0,
      FERTILE_SOIL: 0,
      PEAT: 0,
      CLOUD: 0,
      CLAY: 0,
      ETHER: 0,
    };
    
    const newParticles: Particle[] = [];
    let nextParticleId = 0; // Reset particle IDs for new random grid

    const totalCells = state.width * state.height;

    // Generate random grid elements (around 30% of total cells with variation)
    const baseTargetGridElements = totalCells * 0.3;
    const gridElementVariation = totalCells * 0.1; // +/- 10% of total cells
    const targetGridElements = Math.floor(baseTargetGridElements + (Math.random() - 0.5) * 2 * gridElementVariation);

    for (let i = 0; i < targetGridElements; i++) {
      const randomX = Math.floor(Math.random() * state.width);
      const randomY = Math.floor(Math.random() * state.height);
      const randomElement = gridElements[Math.floor(Math.random() * gridElements.length)];

      if (newGrid[randomY][randomX].type === 'EMPTY') { // Only place if cell is empty
        newGrid[randomY][randomX] = { type: randomElement };
        const baseColor = ELEMENTS[randomElement].color;
        newColorGrid[randomY][randomX] = varyColor(baseColor);
        stats[randomElement]++;
      }
    }

    // Generate random particles (around 1% of total cells with variation)
    const baseTargetParticles = totalCells * 0.01;
    const particleVariation = totalCells * 0.005; // +/- 0.5% of total cells
    const targetTotalParticles = Math.floor(baseTargetParticles + (Math.random() - 0.5) * 2 * particleVariation);

    for (let i = 0; i < targetTotalParticles; i++) {
      const randomX = Math.floor(Math.random() * state.width);
      const randomY = Math.floor(Math.random() * state.height);
      const randomParticleType = particleElementsForRandom[Math.floor(Math.random() * particleElementsForRandom.length)];
      if (!randomParticleType) continue;
      newParticles.push({
        id: nextParticleId++,
        px: randomX + 0.5,
        py: randomY + 0.5,
        vx: (Math.random() - 0.5) * 0.5,
        vy: (Math.random() - 0.5) * 0.5,
        type: randomParticleType,
        life: (ELEMENTS[randomParticleType as ElementName] as any)?.lifespan || 100,
      });
      // stats[randomParticleType]++; // Update stats for particles
    }

    return { grid: newGrid, lastMoveGrid: newLastMoveGrid, colorGrid: newColorGrid, stats, particles: newParticles, nextParticleId };
  }),
  updateStats: (stats) => set({ stats }),
  setFps: (fps) => set({ fps }),
  loadTransformationRules: async () => {
    try {
      const response = await fetch('/rules.json');
      if (!response.ok) {
        throw new Error(`Failed to fetch rules: ${response.statusText}`);
      }
      const rules = await response.json();
      set({ transformationRules: rules });
      console.log('Transformation rules loaded successfully.', rules);
    } catch (error) {
      console.error("Error loading transformation rules:", error);
    }
  },
}));

export default useGameStore;